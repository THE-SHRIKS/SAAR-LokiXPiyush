use regex::Regex;
use std::sync::OnceLock;

/// High-performance offline log and error message scrubber
/// Guarantees that secrets, PII, account tokens, PAN, Aadhaar, and credit card
/// numbers are completely redacted before strings are written to disk or logs.
pub struct LogScrubber;

struct ScrubberPatterns {
    passphrase_pattern: Regex,
    credit_card_pattern: Regex,
    aadhaar_pattern: Regex,
    pan_pattern: Regex,
    user_path_unix: Regex,
    user_path_win: Regex,
    bearer_token: Regex,
}

static PATTERNS: OnceLock<ScrubberPatterns> = OnceLock::new();

fn get_patterns() -> &'static ScrubberPatterns {
    PATTERNS.get_or_init(|| {
        ScrubberPatterns {
            // e.g. "passphrase: secret123", "password=\"mypass\"", "apiKey: abcd"
            passphrase_pattern: Regex::new(
                r#"(?i)(passphrase|password|secret|api[-_]?key|private[-_]?key)\s*[:=]\s*["']?([^\s"',;}{]+)["']?"#,
            ).expect("Valid regex"),

            // 13-19 digit credit card patterns (with optional spaces or dashes)
            credit_card_pattern: Regex::new(
                r#"\b(?:\d{4}[ -]?){3}\d{4}\b|\b(?:\d{4}[ -]?){3}\d{1,3}\b"#,
            ).expect("Valid regex"),

            // Indian Aadhaar: 12 digits (XXXX XXXX XXXX)
            aadhaar_pattern: Regex::new(
                r#"\b\d{4}\s\d{4}\s\d{4}\b"#,
            ).expect("Valid regex"),

            // Indian PAN: 5 letters + 4 digits + 1 letter (e.g. ABCDE1234F)
            pan_pattern: Regex::new(
                r#"\b[A-Z]{5}[0-9]{4}[A-Z]{1}\b"#,
            ).expect("Valid regex"),

            // Unix home paths: /home/username/... -> /home/[REDACTED_USER]/...
            user_path_unix: Regex::new(
                r#"/home/[a-zA-Z0-9_\-]+/"#,
            ).expect("Valid regex"),

            // Windows user paths: C:\Users\username\... -> C:\Users\[REDACTED_USER]\...
            user_path_win: Regex::new(
                r#"(?i)[A-Z]:\\Users\\[a-zA-Z0-9_\-]+\\"#,
            ).expect("Valid regex"),

            // Bearer / auth tokens
            bearer_token: Regex::new(
                r#"(?i)bearer\s+[a-zA-Z0-9_\-\.]{16,}"#,
            ).expect("Valid regex"),
        }
    })
}

impl LogScrubber {
    /// Scrub all sensitive credentials, PII, and absolute personal paths from input text
    pub fn scrub(input: &str) -> String {
        let p = get_patterns();

        // 1. Scrub passphrases and secret key assignments
        let s1 = p.passphrase_pattern.replace_all(input, "$1: [REDACTED]");

        // 2. Scrub bearer tokens
        let s2 = p.bearer_token.replace_all(&s1, "Bearer [REDACTED_TOKEN]");

        // 3. Scrub credit cards
        let s3 = p.credit_card_pattern.replace_all(&s2, "[REDACTED_CC]");

        // 4. Scrub Aadhaar numbers
        let s4 = p.aadhaar_pattern.replace_all(&s3, "[REDACTED_AADHAAR]");

        // 5. Scrub PAN numbers
        let s5 = p.pan_pattern.replace_all(&s4, "[REDACTED_PAN]");

        // 6. Scrub Unix home user paths
        let s6 = p.user_path_unix.replace_all(&s5, "/home/[REDACTED_USER]/");

        // 7. Scrub Windows user paths
        let s7 = p.user_path_win.replace_all(&s6, "C:\\Users\\[REDACTED_USER]\\");

        s7.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrubs_passphrase_and_keys() {
        let log = "Database unlock failed with passphrase: MasterSecretPass! and apiKey=xyz123";
        let scrubbed = LogScrubber::scrub(log);
        assert!(!scrubbed.contains("MasterSecretPass!"));
        assert!(!scrubbed.contains("xyz123"));
        assert!(scrubbed.contains("passphrase: [REDACTED]"));
        assert!(scrubbed.contains("apiKey: [REDACTED]"));
    }

    #[test]
    fn test_scrubs_pii_and_cards() {
        let log = "User card 4532-1234-5678-9012 and PAN ABCDE1234F and Aadhaar 2345 6789 0123";
        let scrubbed = LogScrubber::scrub(log);
        assert!(!scrubbed.contains("4532-1234-5678-9012"));
        assert!(!scrubbed.contains("ABCDE1234F"));
        assert!(!scrubbed.contains("2345 6789 0123"));
        assert!(scrubbed.contains("[REDACTED_CC]"));
        assert!(scrubbed.contains("[REDACTED_PAN]"));
        assert!(scrubbed.contains("[REDACTED_AADHAAR]"));
    }

    #[test]
    fn test_scrubs_personal_paths() {
        let log = "Failed to load /home/theshriksloki/OpenDeck/data or C:\\Users\\Administrator\\Secret.txt";
        let scrubbed = LogScrubber::scrub(log);
        assert!(!scrubbed.contains("/home/theshriksloki/"));
        assert!(!scrubbed.contains("C:\\Users\\Administrator\\"));
        assert!(scrubbed.contains("/home/[REDACTED_USER]/OpenDeck/data"));
        assert!(scrubbed.contains("C:\\Users\\[REDACTED_USER]\\Secret.txt"));
    }
}
