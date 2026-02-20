//! Shared SSE streaming utilities used by all LLM providers.
//!
//! Each provider has different SSE event formats, but the skeleton extraction
//! and partial JSON parsing logic is identical.

/// Parse complete SSE events from a buffer.
///
/// SSE events are separated by `\n\n`. Returns (event_type, data) pairs.
/// Events without an `event:` prefix use empty string as event_type.
/// Removes processed events from the buffer.
pub fn parse_sse_events(buffer: &mut String) -> Vec<(String, String)> {
    let mut events = Vec::new();

    while let Some(pos) = buffer.find("\n\n") {
        let event_block = buffer[..pos].to_string();
        *buffer = buffer[pos + 2..].to_string();

        let mut event_type = String::new();
        let mut data = String::new();

        for line in event_block.lines() {
            if let Some(val) = line.strip_prefix("event: ") {
                event_type = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("data: ") {
                data = val.to_string();
            }
        }

        if !event_type.is_empty() || !data.is_empty() {
            events.push((event_type, data));
        }
    }

    events
}

/// Parse SSE events that have no `event:` prefix (Gemini format).
///
/// Returns just the data payloads.
pub fn parse_data_only_sse_events(buffer: &mut String) -> Vec<String> {
    let mut events = Vec::new();

    while let Some(pos) = buffer.find("\n\n") {
        let event_block = buffer[..pos].to_string();
        *buffer = buffer[pos + 2..].to_string();

        for line in event_block.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if !data.is_empty() {
                    events.push(data.to_string());
                }
            }
        }
    }

    events
}

/// Strip markdown code fences from LLM response text.
///
/// Claude often wraps JSON in ```json ... ``` despite being told not to.
/// Gemini with responseMimeType doesn't need this, but it's safe to call.
pub fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with("```") {
        let after_open = match trimmed.find('\n') {
            Some(pos) => &trimmed[pos + 1..],
            None => trimmed,
        };
        let stripped = after_open.trim_end();
        if stripped.ends_with("```") {
            stripped[..stripped.len() - 3].trim().to_string()
        } else {
            after_open.trim().to_string()
        }
    } else {
        trimmed.to_string()
    }
}

/// Try to extract contentType and summary from partially accumulated JSON.
///
/// Works by finding the first '{' and looking for complete string values.
/// Returns None if either value isn't complete yet (still streaming).
pub fn try_extract_skeleton(accumulated: &str) -> Option<(String, String)> {
    let json_start = accumulated.find('{')?;
    let json_text = &accumulated[json_start..];

    let content_type = extract_json_string_value(json_text, "contentType")?;
    let summary = extract_json_string_value(json_text, "summary")?;
    Some((content_type, summary))
}

/// Extract a string value for a given key from partial JSON text.
///
/// Handles escaped quotes correctly. Returns None if the key isn't found
/// or the closing quote hasn't been streamed yet.
pub fn extract_json_string_value(text: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let key_pos = text.find(&pattern)?;
    let rest = &text[key_pos + pattern.len()..];

    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;

    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(rest[..i].to_string());
        }
        i += 1;
    }

    None
}
