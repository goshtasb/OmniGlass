import { setupOverlay } from "./overlay";

// The app starts hidden (lives in the system tray).
// When the tray icon is clicked, Rust creates an overlay window
// and emits a "screenshot-ready" event with the base64 PNG.
// This file bootstraps the overlay UI in that window.

const app = document.querySelector<HTMLDivElement>("#app")!;
app.innerHTML = `<canvas id="overlay-canvas"></canvas>`;

setupOverlay();
