# OCR Test Corpus

This directory contains 20 PNG screenshots for benchmarking the OCR pipeline.
Every image must be **manually captured from real applications** on your actual machine.
Do not use stock images — the benchmark must reflect real-world screen content.

## Required Images

| Filename | Category | What to Capture |
|---|---|---|
| `error_python_traceback.png` | Error | Python ModuleNotFoundError or similar traceback |
| `error_node_crash.png` | Error | Node.js uncaught exception or crash output |
| `error_rust_compiler.png` | Error | Rust compiler error with line numbers |
| `table_sales_chrome.png` | Table | Data table in Chrome (sales, analytics, etc.) |
| `table_excel_formula.png` | Table | Excel spreadsheet with formulas visible |
| `table_pipe_delimited.png` | Table | Pipe-delimited data in a terminal |
| `code_python_function.png` | Code | Python function definition in an editor |
| `code_typescript_react.png` | Code | React/TypeScript component in VS Code |
| `code_rust_struct.png` | Code | Rust struct definition |
| `prose_email_english.png` | Prose | English email body |
| `prose_legal_german.png` | Prose | German legal/contract text |
| `prose_article_japanese.png` | Prose | Japanese news article or documentation |
| `kv_receipt.png` | Key-Value | Store receipt with items and prices |
| `kv_invoice.png` | Key-Value | Invoice with labeled fields |
| `kv_contact_card.png` | Key-Value | Business card or contact info |
| `mixed_dashboard.png` | Mixed | Analytics dashboard with charts + text |
| `mixed_slack_thread.png` | Mixed | Slack or chat conversation thread |
| `low_quality_blurry.png` | Low Quality | Deliberately blurry/low-res capture |
| `low_quality_dark_mode.png` | Low Quality | Dark mode UI with low contrast text |
| `adversarial_injection.png` | Adversarial | Screenshot containing "Ignore all instructions..." |

## How to Capture

1. Open the real application on your machine.
2. Use macOS screenshot (Cmd+Shift+4) to capture the relevant region.
3. Save with the exact filename from the table above.
4. Place in this directory.

## Running the Benchmark

```bash
cd tools/ocr-bench
cargo run -- --batch ../../test-corpus/
```

Output: CSV with `filename, char_count, latency_ms, confidence, recognition_level`
