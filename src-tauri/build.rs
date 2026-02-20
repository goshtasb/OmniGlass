//! Build script for Omni-Glass Tauri app.
//!
//! Platform-conditional build:
//! 1. Tauri build (generates Tauri-specific code) — all platforms
//! 2. macOS: swift-bridge FFI glue, compile Swift OCR bridge, link frameworks
//! 3. Windows: no extra build steps (windows-rs WinRT bindings are auto-generated)
//!
//! All generated files go to OUT_DIR (inside target/) to avoid triggering
//! Tauri's file watcher on every build.
//!
//! IMPORTANT: The Swift bridge code is behind #[cfg(target_os = "macos")].
//! A runtime `if` check is NOT sufficient — the compiler must not see the
//! swift_bridge_build symbol at all on non-macOS hosts, because the crate
//! isn't a build-dependency on those platforms.

fn main() {
    // Phase 1: Tauri (all platforms)
    tauri_build::build();

    // Phase 2: macOS-only Swift OCR bridge
    #[cfg(target_os = "macos")]
    build_swift_ocr_bridge();
}

/// Build the Swift OCR bridge for macOS.
///
/// Uses swift-bridge to generate Rust↔Swift FFI glue, compiles the Swift
/// source into a static library, and links it with Apple frameworks.
///
/// Gated with #[cfg(target_os = "macos")] so the compiler never resolves
/// swift_bridge_build on Windows/Linux hosts.
#[cfg(target_os = "macos")]
fn build_swift_ocr_bridge() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let swift_src_dir = manifest_dir.join("swift-src");
    let generated_dir = out_dir.join("swift-bridge-generated");

    println!("cargo:rerun-if-changed=src/ocr/apple_vision.rs");
    println!("cargo:rerun-if-changed=swift-src/ocr_bridge.swift");

    // Step 1: Generate FFI glue to OUT_DIR (not inside src-tauri/)
    swift_bridge_build::parse_bridges(vec!["src/ocr/apple_vision.rs"])
        .write_all_concatenated(&generated_dir, env!("CARGO_PKG_NAME"));

    // Step 2: Generate bridging header dynamically with absolute paths
    let bridging_header = out_dir.join("bridging-header.h");
    std::fs::write(
        &bridging_header,
        format!(
            "#ifndef BridgingHeader_h\n\
             #define BridgingHeader_h\n\
             #include \"{generated}/SwiftBridgeCore.h\"\n\
             #include \"{generated}/omni-glass/omni-glass.h\"\n\
             #endif\n",
            generated = generated_dir.display(),
        ),
    )
    .expect("Failed to write bridging header");

    // Step 3: Compile Swift → static library in OUT_DIR
    let lib_output = out_dir.join("libocr_swift.a");

    let status = std::process::Command::new("swiftc")
        .args(["-emit-library", "-static"])
        .args(["-module-name", "ocr_swift"])
        .arg("-import-objc-header")
        .arg(&bridging_header)
        .arg(swift_src_dir.join("ocr_bridge.swift"))
        .arg(generated_dir.join("SwiftBridgeCore.swift"))
        .arg(generated_dir.join("omni-glass/omni-glass.swift"))
        .arg("-o")
        .arg(&lib_output)
        .arg("-O")
        .status()
        .expect("Failed to run swiftc — is Xcode Command Line Tools installed?");

    if !status.success() {
        panic!("swiftc compilation failed");
    }

    // Step 4: Link the static library + macOS frameworks
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=ocr_swift");

    // Apple frameworks required for Vision OCR
    println!("cargo:rustc-link-lib=framework=Vision");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=ImageIO");

    // Swift runtime search paths
    let xcode_path = std::process::Command::new("xcode-select")
        .arg("--print-path")
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().to_string())
        .unwrap_or_else(|_| "/Applications/Xcode.app/Contents/Developer".to_string());

    println!(
        "cargo:rustc-link-search={}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx/",
        xcode_path
    );
    println!("cargo:rustc-link-search=/usr/lib/swift");
}
