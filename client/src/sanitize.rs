/// Sanitises untrusted strings before they are written to the log.
///
/// Removes characters that could be used for log-injection or terminal-escape
/// attacks:
/// - `\n` / `\r` → replaced with a space (prevents fake log lines)
/// - ANSI CSI escape sequences (`ESC [` … final-byte) → stripped entirely
/// - Other C0/C1 control characters (U+0000–U+001F, U+007F), except `\t` → dropped
/// - All printable characters and `\t` are preserved unchanged.
pub fn sanitize_for_logging(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\n' | '\r' => out.push(' '),
            '\x1B' => {
                // ANSI CSI sequence: ESC '[' <params> <final-byte A-Za-z>
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                } else {
                    // Other two-byte escape: skip the following character.
                    chars.next();
                }
            }
            c if c.is_control() && c != '\t' => { /* drop */ }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newlines_become_spaces() {
        assert_eq!(
            sanitize_for_logging("line1\nline2\rline3"),
            "line1 line2 line3"
        );
    }

    #[test]
    fn ansi_color_stripped() {
        assert_eq!(
            sanitize_for_logging("\x1B[31mred\x1B[0mnormal"),
            "rednormal"
        );
    }

    #[test]
    fn ansi_cursor_stripped() {
        assert_eq!(
            sanitize_for_logging("text\x1B[2Jclear\x1B[H"),
            "textclear"
        );
    }

    #[test]
    fn control_chars_dropped() {
        assert_eq!(
            sanitize_for_logging("text\x00null\x07bell"),
            "textnullbell"
        );
    }

    #[test]
    fn tab_preserved() {
        assert_eq!(
            sanitize_for_logging("col1\tcol2"),
            "col1\tcol2"
        );
    }

    #[test]
    fn normal_text_unchanged() {
        assert_eq!(
            sanitize_for_logging("Hello, World! 123"),
            "Hello, World! 123"
        );
    }

    #[test]
    fn log_injection_blocked() {
        let evil = "kicked\n[ERROR] Fake error\n[ERROR] Another fake";
        let s = sanitize_for_logging(evil);
        assert!(!s.contains('\n'));
        assert_eq!(s, "kicked [ERROR] Fake error [ERROR] Another fake");
    }

    #[test]
    fn terminal_escape_blocked() {
        let evil = "kicked\x1B[2J\x1B[HFake login prompt";
        let s = sanitize_for_logging(evil);
        assert!(!s.contains('\x1B'));
        assert_eq!(s, "kickedFake login prompt");
    }
}
