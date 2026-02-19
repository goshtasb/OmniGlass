//! System tray setup and click handler.
//!
//! The tray icon is the primary entry point for Omni-Glass.
//! Clicking it triggers the screen capture flow.

use tauri::{
    image::Image as TauriImage,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

/// Sets up the system tray icon with a click handler.
///
/// Left-click: triggers screen capture (snip mode).
/// Right-click: opens context menu with Quit option.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItemBuilder::with_id("quit", "Quit Omni-Glass").build(app)?;
    let menu = MenuBuilder::new(app).item(&quit_item).build()?;

    // Decode the PNG icon to RGBA for Tauri's Image type
    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon_img = image::load_from_memory(icon_bytes)
        .map_err(|e| format!("Failed to decode tray icon: {}", e))?;
    let rgba = icon_img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let tray_icon = TauriImage::new_owned(rgba.into_raw(), w, h);

    let _tray = TrayIconBuilder::new()
        .icon(tray_icon)
        .tooltip("Omni-Glass — Click to snip")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray_icon, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                log::info!("Tray icon clicked — starting capture");
                let app = tray_icon.app_handle();
                if let Err(e) = start_snip_mode(app) {
                    log::error!("Failed to start snip mode: {}", e);
                }
            }
        })
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                log::info!("Quit requested from tray menu");
                app.exit(0);
            }
        })
        .build(app)?;

    Ok(())
}

/// Initiates snip mode: captures the screen, then opens the overlay window.
fn start_snip_mode(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use crate::capture::{self, CaptureState};
    use base64::{engine::general_purpose::STANDARD, Engine};

    let start = std::time::Instant::now();

    // Step 1: Capture the full screen
    let screenshot = capture::capture_primary_monitor()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    let capture_ms = start.elapsed().as_millis();
    log::info!("Screen captured in {}ms", capture_ms);

    // Step 2: Encode as base64 PNG for the frontend overlay
    let mut png_bytes: Vec<u8> = Vec::new();
    screenshot
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("PNG encoding failed: {}", e))?;
    let base64_png = STANDARD.encode(&png_bytes);

    let encode_ms = start.elapsed().as_millis() - capture_ms;
    log::info!("PNG encoded in {}ms ({} bytes)", encode_ms, png_bytes.len());

    // Step 3: Store the screenshot for later cropping
    let state = app.state::<CaptureState>();
    *state.screenshot.lock().unwrap() = Some(screenshot);

    // Step 4: Create the overlay window
    let overlay_window = tauri::WebviewWindowBuilder::new(
        app,
        "overlay",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .fullscreen(true)
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .title("Omni-Glass Overlay")
    .build()?;

    // Step 5: Send the screenshot to the overlay window via event
    overlay_window.emit("screenshot-ready", &base64_png)?;

    let total_ms = start.elapsed().as_millis();
    log::info!("Overlay opened in {}ms total", total_ms);

    Ok(())
}
