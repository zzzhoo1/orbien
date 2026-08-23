/// Sanitizes untrusted strings for safe logging by removing control characters
/// that could be used for log injection or terminal escape sequence attacks.
///
/// This function:
/// - Replaces newlines (\n, \r) with spaces to prevent log forgery
/// - Removes ANSI escape sequences (ESC followed by control codes)
/// - Removes other control characters (0x00-0x1F, 0x7F) except tab
/// - Preserves printable characters and basic whitespace
pub fn sanitize_for_logging(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            // Replace newlines with space to prevent log forgery
            '\n' | '\r' => result.push(' '),
            
            // ESC character - check if it's the start of an ANSI escape sequence
            '\x1B' => {
                // Skip the entire ANSI escape sequence
                // Common patterns: ESC[...m (SGR), ESC[...H (cursor), etc.
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Skip until we find a letter (the final byte of CSI sequence)
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        if next_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                } else {
                    // Skip other escape sequences (ESC followed by single char)
                    chars.next();
                }
            }
            
            // Remove other control characters except tab
            ch if ch.is_control() && ch != '\t' => {
                // Skip control character
            }
            
            // Keep printable characters and tab
            _ => result.push(ch),
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_newlines() {
        assert_eq!(
            sanitize_for_logging("line1\nline2\rline3"),
            "line1 line2 line3"
        );
    }

    #[test]
    fn test_sanitize_ansi_colors() {
        assert_eq!(
            sanitize_for_logging("normal\x1B[31mred\x1B[0mnormal"),
            "normalrednormal"
        );
    }

    #[test]
    fn test_sanitize_ansi_cursor() {
        assert_eq!(
            sanitize_for_logging("text\x1B[2Jclear\x1B[H"),
            "textclear"
        );
    }

    #[test]
    fn test_sanitize_control_chars() {
        assert_eq!(
            sanitize_for_logging("text\x00null\x07bell"),
            "textnullbell"
        );
    }

    #[test]
    fn test_preserve_tab() {
        assert_eq!(
            sanitize_for_logging("col1\tcol2\tcol3"),
            "col1\tcol2\tcol3"
        );
    }

    #[test]
    fn test_preserve_normal_text() {
        assert_eq!(
            sanitize_for_logging("Hello, World! 123"),
            "Hello, World! 123"
        );
    }

    #[test]
    fn test_log_injection_attack() {
        let malicious = "kicked\n[ERROR] Fake error message\n[ERROR] Another fake";
        let sanitized = sanitize_for_logging(malicious);
        assert!(!sanitized.contains('\n'));
        assert_eq!(sanitized, "kicked [ERROR] Fake error message [ERROR] Another fake");
    }

    #[test]
    fn test_terminal_escape_attack() {
        // Attempt to clear screen and move cursor
        let malicious = "kicked\x1B[2J\x1B[HFake login prompt";
        let sanitized = sanitize_for_logging(malicious);
        assert!(!sanitized.contains('\x1B'));
        assert_eq!(sanitized, "kickedFake login prompt");
    }
}
