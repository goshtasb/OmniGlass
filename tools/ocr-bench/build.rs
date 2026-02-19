fn main() {
    // No-op build script.
    // The Swift OCR helper is compiled at runtime by main.rs
    // using `swiftc` when first needed.
    //
    // For production (Week 2), this will be replaced with
    // swift-bridge-build integration for direct Rust↔Swift FFI.
}
