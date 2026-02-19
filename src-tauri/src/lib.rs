//! Omni-Glass — Tauri application entry point.
//!
//! This is the app shell that wires together:
//! - System tray (tray.rs)
//! - Screen capture domain (capture/)
//! - Tauri command handlers for frontend communication

mod capture;
mod tray;

use capture::CaptureState;
use tauri::Manager;

/// Tauri command: crop the stored screenshot to the given rectangle.
///
/// Called by the frontend overlay when the user releases the mouse.
/// Returns base64-encoded PNG of the cropped region.
#[tauri::command]
fn crop_region(
    state: tauri::State<'_, CaptureState>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let start = std::time::Instant::now();

    let guard = state.screenshot.lock().map_err(|e| e.to_string())?;
    let screenshot = guard
        .as_ref()
        .ok_or("No screenshot available — capture first")?;

    let png_bytes = capture::crop_to_png_bytes(screenshot, x, y, width, height)
        .map_err(|e| e.to_string())?;

    let base64_png = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    );

    let crop_ms = start.elapsed().as_millis();
    log::info!(
        "Cropped region ({}x{} at {},{}) in {}ms — {} bytes",
        width, height, x, y, crop_ms, png_bytes.len()
    );

    Ok(base64_png)
}

/// Tauri command: close the overlay and clean up capture state.
#[tauri::command]
fn close_overlay(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Entry point — called by Tauri runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(CaptureState::new())
        .invoke_handler(tauri::generate_handler![crop_region, close_overlay])
        .setup(|app| {
            log::info!("Omni-Glass starting up");

            tray::setup_tray(app.handle())?;

            log::info!("System tray initialized — ready for snips");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error running Omni-Glass");
}
