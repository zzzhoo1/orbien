use anyhow::{anyhow, bail, Result};
use std::borrow::Cow;

const OPEN: &str = "{{";
const CLOSE: &str = "}}";
const ENV_PREFIX: &str = "env.";

struct Placeholder<'a> {
    end: usize,
    name: &'a str,
    default: Option<&'a str>,
}

enum OpenParse<'a> {
    NotOurs,
    Found(Placeholder<'a>),
}

pub fn expand_env_placeholders(input: &str) -> Result<Cow<'_, str>> {
    if !input.contains(OPEN) {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut copy_from = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            match parse_open(input, i)? {
                OpenParse::NotOurs => i += 1,
                OpenParse::Found(ph) => {
                    out.push_str(&input[copy_from..i]);
                    append_resolved(&mut out, &ph)?;
                    copy_from = ph.end;
                    i = ph.end;
                    continue;
                }
            }
        } else {
            i += 1;
        }
    }
    out.push_str(&input[copy_from..]);
    Ok(Cow::Owned(out))
}

pub fn contains_env_placeholders(input: &str) -> bool {
    let mut i = 0;
    while let Some(rel) = input[i..].find(OPEN) {
        let open = i + rel;
        let j = skip_ws(input, open + OPEN.len());
        if input[j..].starts_with(ENV_PREFIX) {
            return true;
        }
        i = open + 1;
    }
    false
}

pub(crate) fn reject_env_placeholders(raw: &str) -> Result<()> {
    if contains_env_placeholders(raw) {
        bail!(
            "environment placeholders like {placeholder} are not supported in the desktop client; \
             fill in actual values, or run the CLI / Docker client",
            placeholder = "{{env.NAME}}",
        );
    }
    Ok(())
}

fn lookup_process_env(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            bail!("environment variable '{name}' is not valid UTF-8")
        }
    }
}

fn append_resolved(out: &mut String, ph: &Placeholder<'_>) -> Result<()> {
    if let Some(value) = lookup_process_env(ph.name)? {
        out.push_str(&value);
        return Ok(());
    }
    if let Some(default) = ph.default {
        out.push_str(default);
        return Ok(());
    }
    Err(anyhow!(
        "environment variable '{}' is not set (referenced as {{{{env.{}}}}})",
        ph.name,
        ph.name
    ))
}

fn parse_open(input: &str, start: usize) -> Result<OpenParse<'_>> {
    debug_assert!(input[start..].starts_with(OPEN));
    let mut i = skip_ws(input, start + OPEN.len());

    if !input[i..].starts_with(ENV_PREFIX) {
        return Ok(OpenParse::NotOurs);
    }
    i += ENV_PREFIX.len();

    let name = parse_ident(input, &mut i)?;
    if name.is_empty() {
        bail!(
            "invalid environment placeholder at byte {start}: missing variable name after 'env.'"
        );
    }
    i = skip_ws(input, i);

    let default = if input.as_bytes().get(i) == Some(&b':') {
        i += 1;
        let close = find_close(input, i).ok_or_else(|| {
            anyhow!("unclosed environment placeholder at byte {start} (missing '}}}}')")
        })?;
        let raw_default = input[i..close].trim();
        i = close;
        Some(raw_default)
    } else {
        None
    };

    i = skip_ws(input, i);
    if !input[i..].starts_with(CLOSE) {
        if default.is_none() && find_close(input, i).is_none() {
            bail!("unclosed environment placeholder at byte {start} (missing '}}}}')");
        }
        let snippet = snippet_at(input, start);
        bail!(
            "invalid environment placeholder {snippet}: expected '}}}}' or ':default}}}}' after the variable name"
        );
    }
    i += CLOSE.len();

    Ok(OpenParse::Found(Placeholder {
        end: i,
        name,
        default,
    }))
}

fn parse_ident<'a>(input: &'a str, i: &mut usize) -> Result<&'a str> {
    let bytes = input.as_bytes();
    let start = *i;
    if start >= bytes.len() {
        return Ok("");
    }
    let first = bytes[start];
    if !is_ident_start(first) {
        if first.is_ascii_alphanumeric() || first == b'_' {
            bail!(
                "invalid environment variable name: must start with a letter or '_', got '{}'",
                first as char
            );
        }
        return Ok("");
    }
    *i += 1;
    while *i < bytes.len() && is_ident_continue(bytes[*i]) {
        *i += 1;
    }
    Ok(&input[start..*i])
}

fn find_close(input: &str, from: usize) -> Option<usize> {
    input[from..].find(CLOSE).map(|rel| from + rel)
}

fn skip_ws(input: &str, mut i: usize) -> usize {
    let bytes = input.as_bytes();
    while i < bytes.len() && is_placeholder_ws(bytes[i]) {
        i += 1;
    }
    i
}

fn is_placeholder_ws(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn snippet_at(input: &str, start: usize) -> String {
    let rest = &input[start..];
    let end = rest.find(CLOSE).map(|i| (i + CLOSE.len()).min(rest.len()));
    let taken = match end {
        Some(e) => &rest[..e],
        None => &rest[..rest.len().min(48)],
    };
    format!("`{taken}`")
}
