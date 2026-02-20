//! Sensitive data redaction — scans OCR text for PII and secrets
//! before sending to cloud LLM providers.
//!
//! Patterns from LLM Integration PRD Section 9.

use regex::Regex;
use std::sync::LazyLock;

pub struct RedactionResult {
    pub cleaned_text: String,
    pub redactions: Vec<Redaction>,
    pub has_redactions: bool,
}

pub struct Redaction {
    pub label: String,
    pub count: usize,
}

static SENSITIVE_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Credit card numbers (4 groups of 4 digits)
        (
            Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap(),
            "credit_card",
        ),
        // SSN (XXX-XX-XXXX)
        (Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(), "ssn"),
        // API keys (common formats: sk-..., pk-..., api-..., etc.)
        (
            Regex::new(r"\b(sk|pk|api|key|token|secret)[-_][a-zA-Z0-9_-]{20,}\b").unwrap(),
            "api_key",
        ),
        // AWS access keys (AKIA + 16 alphanumeric)
        (Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(), "aws_key"),
        // Private key blocks
        (
            Regex::new(r"-----BEGIN (RSA |EC |DSA )?PRIVATE KEY-----").unwrap(),
            "private_key",
        ),
    ]
});

/// Scan text for sensitive data patterns and replace matches with
/// `[REDACTED:<label>]` placeholders.
///
/// Returns the cleaned text and a summary of what was redacted.
pub fn redact_sensitive_data(text: &str) -> RedactionResult {
    let mut cleaned = text.to_string();
    let mut redactions = Vec::new();

    for (pattern, label) in SENSITIVE_PATTERNS.iter() {
        let matches: Vec<_> = pattern.find_iter(&cleaned).collect();
        if !matches.is_empty() {
            redactions.push(Redaction {
                label: label.to_string(),
                count: matches.len(),
            });
            cleaned = pattern
                .replace_all(&cleaned, format!("[REDACTED:{}]", label).as_str())
                .to_string();
        }
    }

    let has_redactions = !redactions.is_empty();

    if has_redactions {
        let summary: Vec<String> = redactions
            .iter()
            .map(|r| format!("{} {}", r.count, r.label))
            .collect();
        log::info!("[SAFETY] Redacted {}", summary.join(", "));
    }

    RedactionResult {
        cleaned_text: cleaned,
        redactions,
        has_redactions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_credit_card() {
        let result = redact_sensitive_data("My card is 4111-1111-1111-1111 thanks");
        assert!(result.has_redactions);
        assert!(result.cleaned_text.contains("[REDACTED:credit_card]"));
        assert!(!result.cleaned_text.contains("4111"));
    }

    #[test]
    fn redacts_ssn() {
        let result = redact_sensitive_data("SSN: 123-45-6789");
        assert!(result.has_redactions);
        assert!(result.cleaned_text.contains("[REDACTED:ssn]"));
        assert!(!result.cleaned_text.contains("123-45-6789"));
    }

    #[test]
    fn redacts_api_key() {
        let result =
            redact_sensitive_data("Use key sk-ant-api03-Gb5s7tjg3Nsw4qOUIWlwUen1TEDdcxd5BEo");
        assert!(result.has_redactions);
        assert!(result.cleaned_text.contains("[REDACTED:api_key]"));
    }

    #[test]
    fn redacts_aws_key() {
        let result = redact_sensitive_data("AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(result.has_redactions);
        assert!(result.cleaned_text.contains("[REDACTED:aws_key]"));
    }

    #[test]
    fn redacts_private_key() {
        let result = redact_sensitive_data("-----BEGIN RSA PRIVATE KEY-----\nMIIE...");
        assert!(result.has_redactions);
        assert!(result.cleaned_text.contains("[REDACTED:private_key]"));
    }

    #[test]
    fn no_redaction_for_clean_text() {
        let result = redact_sensitive_data("Just a normal error message: ModuleNotFoundError");
        assert!(!result.has_redactions);
        assert_eq!(
            result.cleaned_text,
            "Just a normal error message: ModuleNotFoundError"
        );
    }

    #[test]
    fn redacts_multiple_patterns() {
        let text = "Card: 4111-1111-1111-1111, SSN: 123-45-6789";
        let result = redact_sensitive_data(text);
        assert!(result.has_redactions);
        assert_eq!(result.redactions.len(), 2);
        assert!(result.cleaned_text.contains("[REDACTED:credit_card]"));
        assert!(result.cleaned_text.contains("[REDACTED:ssn]"));
    }
}
