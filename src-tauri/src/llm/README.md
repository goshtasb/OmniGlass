# llm/ — Multi-Provider LLM Classification & Execution

## Overview

The LLM module is the "brain" of Omni-Glass. It takes OCR-extracted text and
determines what the user can do with it (CLASSIFY), then performs the chosen
action (EXECUTE). It supports multiple providers (Anthropic Claude, Google Gemini)
with streaming responses for low perceived latency. The classify step emits
partial results to the action menu as SSE chunks arrive.

## Public API

| Export | Type | Description |
|---|---|---|
| `classify_streaming(app, text, ...)` | Async fn | Stream-classify via Anthropic Claude, emits skeleton + complete events |
| `classify_streaming_gemini(app, text, ...)` | Async fn | Stream-classify via Google Gemini Flash |
| `execute_action_anthropic(action_id, text)` | Async fn | Execute a chosen action via Claude, returns `ActionResult` |
| `ActionMenu` | Struct | Full classification result: summary, content_type, actions list |
| `ActionMenuSkeleton` | Struct | Partial result emitted at TTFT: content_type + summary |
| `ActionResult` | Struct | Execution result: status, result body, optional metadata |
| `ActionMenuState` | Struct | Thread-safe storage for menu + OCR text + crop PNG bytes |
| `provider::all_providers()` | Function | List all supported providers with metadata |
| `provider::is_provider_configured(id)` | Function | Check if a provider has an API key available |

## Internal Structure

| File | Lines | Responsibility |
|---|---|---|
| `mod.rs` | 47 | Public re-exports, `ActionMenuState` definition |
| `classify.rs` | 342 | Anthropic Claude streaming classify pipeline |
| `execute.rs` | 293 | Anthropic Claude execute pipeline + JSON salvage |
| `gemini.rs` | 241 | Google Gemini streaming classify pipeline |
| `prompts.rs` | 100 | CLASSIFY system prompt, model constant, token limits |
| `prompts_execute.rs` | 151 | EXECUTE system prompt, per-action templates |
| `streaming.rs` | 122 | SSE event parsing, partial JSON extraction, code fence stripping |
| `types.rs` | 79 | `ActionMenu`, `Action`, `ActionMenuSkeleton` type definitions |
| `provider.rs` | 52 | Provider metadata, configuration checks |

## Dependencies

| Crate / Module | Used For |
|---|---|
| `reqwest` | HTTP client for Anthropic and Gemini APIs |
| `serde` / `serde_json` | JSON serialization/deserialization |
| `tauri::Emitter` | Emit streaming events to frontend windows |
| `crate::safety` | PII redaction before API calls, command safety checks after |

## Used By

| Module | Imports | Purpose |
|---|---|---|
| `pipeline.rs` | `classify_streaming`, `execute_action_anthropic`, `ActionMenuState` | Core snip-to-action flow |
| `commands.rs` | `ActionMenuState`, `ActionMenu` | Serve menu data to frontend |
| `settings_commands.rs` | `provider::all_providers`, `provider::is_provider_configured` | Settings panel provider list |

## Two-Phase LLM Flow

```
CLASSIFY (streaming)              EXECUTE (non-streaming)
OCR text ──→ action menu          OCR text + action_id ──→ ActionResult
  emits skeleton at TTFT            returns full JSON when done
  emits complete when parsed        supports JSON salvage for truncated responses
```

## Architecture Decisions

- **Streaming classify, non-streaming execute**: Classify streams because the user
  is waiting and sees progressive updates. Execute doesn't stream because the user
  already clicked a button and expects a brief wait.
- **JSON salvage**: When `max_tokens` truncates the response, `extract_json_string_field`
  manually parses key-value pairs from malformed JSON rather than failing entirely.
- **Dual-mode fix prompt**: `PROMPT_SUGGEST_FIX` auto-detects environment fixes
  (returns `type: "command"`) vs code fixes (returns `type: "text"` with corrected code).
- **Pre-flight redaction**: All OCR text passes through `safety::redact` before
  being sent to any cloud API.
