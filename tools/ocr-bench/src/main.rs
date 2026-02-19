//! OCR Benchmark CLI for Omni-Glass.
//!
//! Benchmarks Apple Vision Framework text recognition via a Swift helper.
//!
//! Usage:
//!   cargo run -- <image.png>              Single image, accurate mode
//!   cargo run -- <image.png> --fast       Single image, fast mode
//!   cargo run -- --batch <directory>      All PNGs in directory → CSV output
//!   cargo run -- --batch <directory> --fast  Batch with fast mode

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  ocr-bench <image.png> [--fast]");
        eprintln!("  ocr-bench --batch <directory> [--fast]");
        std::process::exit(1);
    }

    let use_fast = args.contains(&"--fast".to_string());

    // Ensure the Swift helper is compiled
    let helper_path = ensure_swift_helper_compiled();

    if args[1] == "--batch" {
        let dir = args.get(2).expect("--batch requires a directory path");
        run_batch(&helper_path, dir, use_fast);
    } else {
        run_single(&helper_path, &args[1], use_fast);
    }
}

/// Compiles the Swift OCR helper if it doesn't exist or is outdated.
fn ensure_swift_helper_compiled() -> PathBuf {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let swift_src = project_root.join("swift-src/ocr.swift");
    let binary = project_root.join("swift-src/ocr-helper");

    let needs_compile = if binary.exists() {
        // Recompile if source is newer than binary
        let src_modified = std::fs::metadata(&swift_src)
            .and_then(|m| m.modified())
            .ok();
        let bin_modified = std::fs::metadata(&binary)
            .and_then(|m| m.modified())
            .ok();
        match (src_modified, bin_modified) {
            (Some(src), Some(bin)) => src > bin,
            _ => true,
        }
    } else {
        true
    };

    if needs_compile {
        eprintln!("Compiling Swift OCR helper...");
        let output = Command::new("swiftc")
            .args(["-O", "-o"])
            .arg(&binary)
            .arg(&swift_src)
            .output()
            .expect("Failed to run swiftc — is Xcode installed?");

        if !output.status.success() {
            eprintln!(
                "Swift compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::process::exit(1);
        }
        eprintln!("Swift helper compiled successfully.");
    }

    binary
}

/// Runs OCR on a single image and prints the result.
fn run_single(helper: &Path, image_path: &str, use_fast: bool) {
    let abs_path = std::fs::canonicalize(image_path).unwrap_or_else(|_| {
        eprintln!("File not found: {}", image_path);
        std::process::exit(1);
    });

    let mut cmd = Command::new(helper);
    cmd.arg(&abs_path);
    if use_fast {
        cmd.arg("--fast");
    }

    let start = Instant::now();
    let output = cmd.output().expect("Failed to execute OCR helper");
    let total_ms = start.elapsed().as_millis();

    if !output.status.success() {
        eprintln!(
            "OCR failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.trim());
    eprintln!("--- Total wall time (including subprocess): {}ms ---", total_ms);
}

/// Runs OCR on all PNG files in a directory, outputs CSV.
fn run_batch(helper: &Path, dir_path: &str, use_fast: bool) {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        eprintln!("Not a directory: {}", dir_path);
        std::process::exit(1);
    }

    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("Failed to read directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map(|ext| ext == "png" || ext == "jpg" || ext == "jpeg")
                .unwrap_or(false)
        })
        .collect();
    entries.sort();

    if entries.is_empty() {
        eprintln!("No image files found in {}", dir_path);
        std::process::exit(1);
    }

    // Print CSV header
    println!("filename,char_count,latency_ms,confidence,recognition_level");

    let mut latencies: Vec<f64> = Vec::new();

    for image_path in &entries {
        let filename = image_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut cmd = Command::new(helper);
        cmd.arg(image_path);
        if use_fast {
            cmd.arg("--fast");
        }

        let output = cmd.output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(result) = parse_ocr_json(&stdout) {
                    println!(
                        "{},{},{:.1},{:.2},{}",
                        filename,
                        result.char_count,
                        result.latency_ms,
                        result.confidence,
                        result.recognition_level
                    );
                    latencies.push(result.latency_ms);
                    std::io::stdout().flush().ok();
                } else {
                    eprintln!("  WARNING: Failed to parse OCR output for {}", filename);
                }
            }
            _ => {
                eprintln!("  WARNING: OCR failed for {}", filename);
            }
        }
    }

    // Print summary statistics
    if !latencies.is_empty() {
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = latencies[latencies.len() / 2];
        let p99_idx = ((latencies.len() as f64 * 0.99).ceil() as usize).min(latencies.len() - 1);
        let p99 = latencies[p99_idx];
        let avg: f64 = latencies.iter().sum::<f64>() / latencies.len() as f64;

        eprintln!("\n--- Benchmark Summary ---");
        eprintln!("  Images processed: {}", latencies.len());
        eprintln!("  Median latency:   {:.1}ms", median);
        eprintln!("  Average latency:  {:.1}ms", avg);
        eprintln!("  P99 latency:      {:.1}ms", p99);
        eprintln!(
            "  Target (< 300ms): {}",
            if median < 300.0 { "PASS ✓" } else { "FAIL ✗" }
        );
    }
}

#[derive(Debug)]
struct OcrResult {
    char_count: usize,
    latency_ms: f64,
    confidence: f64,
    recognition_level: String,
}

/// Parses the JSON output from the Swift helper.
fn parse_ocr_json(json_str: &str) -> Option<OcrResult> {
    // Simple JSON parsing without pulling in serde for the benchmark tool.
    // The Swift helper outputs: { "text": "...", "charCount": N, "latencyMs": F, ... }
    let trimmed = json_str.trim();

    let char_count = extract_json_int(trimmed, "charCount")?;
    let latency_ms = extract_json_float(trimmed, "latencyMs")?;
    let confidence = extract_json_float(trimmed, "confidence").unwrap_or(0.0);
    let recognition_level =
        extract_json_string(trimmed, "recognitionLevel").unwrap_or_else(|| "unknown".to_string());

    Some(OcrResult {
        char_count: char_count as usize,
        latency_ms,
        confidence,
        recognition_level,
    })
}

fn extract_json_int(json: &str, key: &str) -> Option<i64> {
    let pattern = format!("\"{}\" : ", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-')?;
    rest[..end].parse().ok()
}

fn extract_json_float(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\" : ", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E')?;
    rest[..end].parse().ok()
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\" : \"", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
