# safety/ — PII Redaction & Command Blocklist

## Overview

The safety module protects users in two ways: it redacts sensitive data (SSNs,
credit cards, API keys, private keys) before any text is sent to cloud LLM APIs,
and it validates shell commands returned by the LLM against a blocklist of
destructive patterns before execution. This is the trust boundary between
user data and the cloud, and between LLM output and the local system.

## Public API

| Export | Type | Description |
|---|---|---|
| `redact::redact_sensitive_data(text)` | Function | Scan text for PII/secrets, replace with `[REDACTED:label]` tokens |
| `redact::RedactionResult` | Struct | `cleaned_text`, `redactions` list, `has_redactions` flag |
| `redact::Redaction` | Struct | `label` (e.g. "ssn"), `count` of occurrences |
| `command_check::is_command_safe(cmd)` | Function | Check a shell command against the blocklist |
| `command_check::CommandCheck` | Struct | `safe: bool`, `reason: Option<String>` |
| `command_check::is_path_safe(path)` | Function | Check a file path for traversal attacks |

## Internal Structure

| File | Lines | Responsibility |
|---|---|---|
| `mod.rs` | 8 | Re-exports `command_check` and `redact` sub-modules |
| `redact.rs` | 143 | Regex-based PII/secret detection and replacement, with unit tests |
| `command_check.rs` | 163 | Command blocklist patterns, path validation, with unit tests |

## Redaction Patterns

| Pattern | Label | Example |
|---|---|---|
| Credit card numbers | `credit_card` | `4111-1111-1111-1111` |
| Social Security Numbers | `ssn` | `123-45-6789` |
| API keys | `api_key` | `sk-ant-api03-...`, `token-...` |
| AWS access keys | `aws_key` | `AKIA...` |
| Private key blocks | `private_key` | `-----BEGIN RSA PRIVATE KEY-----` |

## Blocked Command Patterns

| Pattern | Example | Reason |
|---|---|---|
| Recursive delete | `rm -rf /` | Filesystem destruction |
| Disk format | `mkfs`, `dd if=` | Data loss |
| Fork bomb | `:(){ :\|:& };:` | System crash |
| Permission escalation | `chmod 777 /` | Security risk |
| Pipe to shell | `curl ... \| sh` | Arbitrary code execution |

## Dependencies

| Crate | Used For |
|---|---|
| `regex` | Pattern matching for PII detection and command validation |

## Used By

| Module | Imports | Purpose |
|---|---|---|
| `llm/execute.rs` | `redact::redact_sensitive_data`, `command_check::is_command_safe`, `command_check::is_path_safe` | Pre-flight redaction, post-flight command/path validation |
| `commands.rs` | `command_check::is_command_safe`, `command_check::is_path_safe` | Validate confirmed commands and file paths |

## Architecture Decisions

- **Regex over ML**: Redaction uses deterministic regex patterns, not ML-based NER.
  This ensures zero false negatives for known patterns (SSN, credit card formats)
  and keeps the module dependency-free beyond `regex`.
- **Defense in depth**: Commands are checked twice — once in `execute.rs` after
  the LLM returns them, and again in `run_confirmed_command` before execution.
  The user also sees a confirmation dialog between these checks.
- **Pure functions**: Both `redact_sensitive_data` and `is_command_safe` are pure
  functions with no I/O. They take a string and return a result. This makes them
  trivially testable (14 unit tests cover both modules).
