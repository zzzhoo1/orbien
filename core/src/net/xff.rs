use anyhow::{bail, Result};

pub fn apply_x_forwarded_for(head: &mut Vec<u8>, client_ip: &str, proto: &str) -> Result<()> {
    let text = String::from_utf8_lossy(head);
    let mut lines: Vec<String> = text.split_inclusive('\n').map(|s| s.to_string()).collect();
    if lines.is_empty() {
        bail!("empty http request");
    }

    let ending = if lines.iter().any(|l| l.ends_with("\r\n")) {
        "\r\n"
    } else {
        "\n"
    };

    let mut xff_idx: Option<usize> = None;
    let mut has_proto = false;
    for (i, line) in lines.iter().enumerate().skip(1) {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let lower_ok_xff = trimmed.len() >= 16
            && trimmed.as_bytes()[..15].eq_ignore_ascii_case(b"x-forwarded-for")
            && trimmed.as_bytes().get(15) == Some(&b':');
        if lower_ok_xff {
            xff_idx = Some(i);
        }
        let lower_ok_proto = trimmed.len() >= 18
            && trimmed.as_bytes()[..17].eq_ignore_ascii_case(b"x-forwarded-proto")
            && trimmed.as_bytes().get(17) == Some(&b':');
        if lower_ok_proto {
            has_proto = true;
        }
    }

    if !client_ip.is_empty() {
        if let Some(i) = xff_idx {
            let line = &lines[i];
            let trimmed = line.trim_start_matches([' ', '\t']);
            let colon = trimmed
                .find(':')
                .ok_or_else(|| anyhow::anyhow!("malformed X-Forwarded-For"))?;
            let value = trimmed[colon + 1..].trim().trim_end_matches(['\r', '\n']);
            let new_val = if value.is_empty() {
                client_ip.to_string()
            } else {
                format!("{value}, {client_ip}")
            };
            lines[i] = format!("X-Forwarded-For: {new_val}{ending}");
        } else {
            let blank_idx = lines
                .iter()
                .position(|l| l == "\r\n" || l == "\n")
                .unwrap_or(lines.len());
            lines.insert(blank_idx, format!("X-Forwarded-For: {client_ip}{ending}"));
        }
    }

    if !proto.is_empty() && !has_proto {
        let blank_idx = lines
            .iter()
            .position(|l| l == "\r\n" || l == "\n")
            .unwrap_or(lines.len());
        lines.insert(blank_idx, format!("X-Forwarded-Proto: {proto}{ending}"));
    }

    *head = lines.join("").into_bytes();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> Vec<u8> {
        b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec()
    }

    #[test]
    fn adds_xff_when_absent() {
        let mut head = req();
        apply_x_forwarded_for(&mut head, "1.2.3.4", "http").unwrap();
        let text = String::from_utf8(head).unwrap();
        assert!(text.contains("X-Forwarded-For: 1.2.3.4\r\n"));
        assert!(text.contains("X-Forwarded-Proto: http\r\n"));
    }

    #[test]
    fn appends_to_existing_xff() {
        let mut head = b"GET / HTTP/1.1\r\nX-Forwarded-For: 9.9.9.9\r\nHost: x\r\n\r\n".to_vec();
        apply_x_forwarded_for(&mut head, "1.2.3.4", "").unwrap();
        let text = String::from_utf8(head).unwrap();
        assert!(text.contains("X-Forwarded-For: 9.9.9.9, 1.2.3.4\r\n"));
    }

    #[test]
    fn does_not_duplicate_proto() {
        let mut head =
            b"GET / HTTP/1.1\r\nX-Forwarded-Proto: https\r\n\r\n".to_vec();
        apply_x_forwarded_for(&mut head, "1.2.3.4", "https").unwrap();
        let text = String::from_utf8(head).unwrap();
        assert_eq!(text.matches("X-Forwarded-Proto").count(), 1);
        assert!(text.contains("X-Forwarded-For: 1.2.3.4\r\n"));
    }

    #[test]
    fn empty_client_ip_adds_nothing() {
        let mut head = req();
        apply_x_forwarded_for(&mut head, "", "").unwrap();
        let text = String::from_utf8(head).unwrap();
        assert!(!text.contains("X-Forwarded"));
    }

    #[test]
    fn supports_lf_only() {
        let mut head = b"GET / HTTP/1.1\nHost: example.com\n\n".to_vec();
        apply_x_forwarded_for(&mut head, "1.2.3.4", "http").unwrap();
        let text = String::from_utf8(head).unwrap();
        assert!(text.contains("X-Forwarded-For: 1.2.3.4\n"));
        assert!(text.contains("X-Forwarded-Proto: http\n"));
    }

    #[test]
    fn empty_request_errors() {
        let mut head: Vec<u8> = vec![];
        assert!(apply_x_forwarded_for(&mut head, "1.2.3.4", "http").is_err());
    }
}
