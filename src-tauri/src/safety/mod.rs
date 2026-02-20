//! Safety layer — sensitive data redaction and command validation.
//!
//! All OCR text passes through redaction before reaching cloud LLMs.
//! All LLM-suggested commands pass through the blocklist before
//! being shown to the user.

pub mod command_check;
pub mod redact;
