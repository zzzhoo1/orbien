use slint::{Model, ModelRc, SharedString, VecModel};
use std::collections::VecDeque;
use std::rc::Rc;

pub const MAX_LOG_LINES: usize = 1_000;
pub type Level = i32;

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: Level,
    pub tag: String,
    pub body: String,
    pub text: String,
}

#[derive(Default)]
pub struct LogStore {
    entries: VecDeque<LogEntry>,
    epoch: u64,
}

#[derive(Default, Clone, Copy)]
pub struct UiSyncCursor {
    pub epoch: u64,
    pub len: usize,
}

impl LogStore {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.epoch = self.epoch.wrapping_add(1);
    }

    pub fn push_raw(&mut self, raw: &str) {
        self.entries.push_back(parse_line(raw));
        self.trim();
    }

    pub fn push_many_raw(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        for raw in lines {
            self.entries.push_back(parse_line(raw));
        }
        self.trim();
    }

    fn trim(&mut self) {
        let mut dropped = false;
        while self.entries.len() > MAX_LOG_LINES {
            self.entries.pop_front();
            dropped = true;
        }
        if dropped {
            self.epoch = self.epoch.wrapping_add(1);
        }
    }

    pub fn to_model(&self, query: &str) -> Rc<VecModel<crate::LogLine>> {
        let rows: Vec<crate::LogLine> = self
            .entries
            .iter()
            .map(|e| entry_to_row(e, query))
            .collect();
        Rc::new(VecModel::from(rows))
    }

    pub fn append_to_model(
        &self,
        model: &VecModel<crate::LogLine>,
        query: &str,
        cursor: UiSyncCursor,
    ) -> Option<UiSyncCursor> {
        if cursor.epoch != self.epoch || model.row_count() != cursor.len {
            return None;
        }
        if cursor.len > self.entries.len() {
            return None;
        }
        for entry in self.entries.iter().skip(cursor.len) {
            model.push(entry_to_row(entry, query));
        }
        Some(UiSyncCursor {
            epoch: self.epoch,
            len: self.entries.len(),
        })
    }

    pub fn cursor_after_full_sync(&self) -> UiSyncCursor {
        UiSyncCursor {
            epoch: self.epoch,
            len: self.entries.len(),
        }
    }
}

pub fn parse_line(raw: &str) -> LogEntry {
    let text = raw.to_string();
    let trimmed = text.trim_start();

    for (name, level) in [
        ("ERROR", 3),
        ("WARN", 2),
        ("WARNING", 2),
        ("INFO", 1),
        ("DEBUG", 4),
        ("TRACE", 4),
    ] {
        if let Some(rest) = strip_level_prefix(trimmed, name) {
            return LogEntry {
                level,
                tag: name.to_string(),
                body: rest,
                text,
            };
        }
    }

    let (tag, body) = match trimmed.find(']') {
        Some(end) if trimmed.starts_with('[') => {
            let tag = trimmed[..=end].to_string();
            let rest = trimmed[end + 1..].to_string();
            (tag, rest)
        }
        _ => (String::new(), trimmed.to_string()),
    };

    LogEntry {
        level: 0,
        tag,
        body,
        text,
    }
}

fn strip_level_prefix(s: &str, level: &str) -> Option<String> {
    if s.len() < level.len() {
        return None;
    }
    let (head, rest) = s.split_at(level.len());
    if !head.eq_ignore_ascii_case(level) {
        return None;
    }
    if rest.is_empty() {
        return Some(String::new());
    }
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(format!("  {}", rest.trim_start()))
}

pub fn highlight_parts(text: &str, query: &str) -> (String, String, String, bool) {
    let q = query.trim();
    if q.is_empty() {
        return (String::new(), String::new(), String::new(), false);
    }
    let lower = text.to_ascii_lowercase();
    let q_lower = q.to_ascii_lowercase();
    if let Some(pos) = lower.find(&q_lower) {
        let end = pos + q_lower.len().min(text.len().saturating_sub(pos));
        let pre = text[..pos].to_string();
        let mid = text[pos..end].to_string();
        let post = text[end..].to_string();
        (pre, mid, post, true)
    } else {
        (String::new(), String::new(), String::new(), false)
    }
}

pub fn make_model() -> Rc<VecModel<crate::LogLine>> {
    Rc::new(VecModel::default())
}

pub fn entry_to_row(entry: &LogEntry, query: &str) -> crate::LogLine {
    let (pre, mid, post, has_hit) = highlight_parts(&entry.text, query);
    crate::LogLine {
        level: entry.level,
        tag: SharedString::from(entry.tag.as_str()),
        body: SharedString::from(entry.body.as_str()),
        text: SharedString::from(entry.text.as_str()),
        pre: SharedString::from(pre),
        mid: SharedString::from(mid),
        post: SharedString::from(post),
        has_hit,
    }
}

pub fn as_vec_model(model: &ModelRc<crate::LogLine>) -> Option<&VecModel<crate::LogLine>> {
    model.as_any().downcast_ref::<VecModel<crate::LogLine>>()
}
