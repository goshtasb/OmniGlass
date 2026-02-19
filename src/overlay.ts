/**
 * Overlay module — handles the fullscreen snip interaction.
 *
 * Flow:
 * 1. Receives a base64 screenshot from Rust via Tauri event.
 * 2. Draws it on a canvas with a 50% dark overlay.
 * 3. User drags a rectangle to select a region.
 * 4. On mouseup, sends the rectangle coordinates to Rust.
 * 5. Rust crops the screenshot and returns the result.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

interface SelectionRect {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

export function setupOverlay(): void {
  const canvas = document.getElementById("overlay-canvas") as HTMLCanvasElement;
  if (!canvas) return;

  const ctx = canvas.getContext("2d")!;
  let screenshotImage: HTMLImageElement | null = null;
  let selection: SelectionRect | null = null;
  let isDragging = false;

  // Resize canvas to fill the screen
  function resizeCanvas(): void {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    if (screenshotImage) {
      drawOverlay();
    }
  }

  // Draw the screenshot with dark overlay and selection rectangle
  function drawOverlay(): void {
    if (!screenshotImage) return;

    // Draw the screenshot scaled to fill the window
    ctx.drawImage(screenshotImage, 0, 0, canvas.width, canvas.height);

    // Dark overlay (50% opacity)
    ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // If there's an active selection, cut through the overlay
    if (selection) {
      const x = Math.min(selection.startX, selection.endX);
      const y = Math.min(selection.startY, selection.endY);
      const w = Math.abs(selection.endX - selection.startX);
      const h = Math.abs(selection.endY - selection.startY);

      if (w > 0 && h > 0) {
        // Clear the dark overlay in the selected region
        ctx.clearRect(x, y, w, h);
        // Redraw the screenshot in the selected region (no dimming)
        ctx.drawImage(
          screenshotImage,
          x, y, w, h,   // Source (screen coords map 1:1 for fullscreen)
          x, y, w, h    // Destination
        );

        // Selection border
        ctx.strokeStyle = "#00b4ff";
        ctx.lineWidth = 2;
        ctx.strokeRect(x, y, w, h);

        // Dimension label
        ctx.fillStyle = "#00b4ff";
        ctx.font = "12px monospace";
        ctx.fillText(`${Math.round(w)} × ${Math.round(h)}`, x, y - 6);
      }
    }
  }

  // Mouse event handlers
  canvas.addEventListener("mousedown", (e: MouseEvent) => {
    isDragging = true;
    selection = {
      startX: e.clientX,
      startY: e.clientY,
      endX: e.clientX,
      endY: e.clientY,
    };
  });

  canvas.addEventListener("mousemove", (e: MouseEvent) => {
    if (!isDragging || !selection) return;
    selection.endX = e.clientX;
    selection.endY = e.clientY;
    drawOverlay();
  });

  canvas.addEventListener("mouseup", async (e: MouseEvent) => {
    if (!isDragging || !selection) return;
    isDragging = false;
    selection.endX = e.clientX;
    selection.endY = e.clientY;

    const x = Math.min(selection.startX, selection.endX);
    const y = Math.min(selection.startY, selection.endY);
    const w = Math.abs(selection.endX - selection.startX);
    const h = Math.abs(selection.endY - selection.startY);

    // Ignore tiny selections (accidental clicks)
    if (w < 10 || h < 10) {
      // Close overlay on click without drag (escape hatch)
      await invoke("close_overlay");
      return;
    }

    console.log(`Selection: ${w}×${h} at (${x}, ${y})`);

    try {
      // Send coordinates to Rust for cropping
      // Scale from CSS pixels to actual screenshot pixels
      const scale = window.devicePixelRatio || 1;
      const croppedBase64: string = await invoke("crop_region", {
        x: Math.round(x * scale),
        y: Math.round(y * scale),
        width: Math.round(w * scale),
        height: Math.round(h * scale),
      });

      console.log(`Cropped region: ${croppedBase64.length} base64 chars`);

      // For the Week 1 spike, just log success and close.
      // Week 2 will pass this to OCR → LLM → action menu.
      await invoke("close_overlay");
    } catch (err) {
      console.error("Crop failed:", err);
      await invoke("close_overlay");
    }
  });

  // Escape key closes the overlay
  document.addEventListener("keydown", async (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      await invoke("close_overlay");
    }
  });

  // Listen for the screenshot from Rust
  listen<string>("screenshot-ready", (event) => {
    console.log("Screenshot received, loading image...");
    const img = new Image();
    img.onload = () => {
      screenshotImage = img;
      resizeCanvas();
      console.log(`Screenshot loaded: ${img.width}×${img.height}`);
    };
    img.src = `data:image/png;base64,${event.payload}`;
  });

  window.addEventListener("resize", resizeCanvas);
  resizeCanvas();
}
