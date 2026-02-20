/**
 * Confirmation dialog — shows a command and explanation before execution.
 *
 * Opened by the action menu when an LLM-suggested command needs user approval.
 * The command and explanation are passed via the window label query params.
 *
 * Flow:
 * 1. Action menu calls execute_action → gets ActionResult with needs_confirmation
 * 2. Action menu opens this window, passing command + explanation via events
 * 3. User clicks "Run" → this window calls run_confirmed_command
 * 4. Result shown briefly, then window closes
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ConfirmPayload {
  command: string;
  explanation: string;
  actionId: string;
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function renderDialog(payload: ConfirmPayload): void {
  const container = document.getElementById("confirm-dialog")!;

  container.innerHTML = `
    <div style="
      background: #1a1a2e;
      border-radius: 8px;
      box-shadow: 0 4px 16px rgba(0,0,0,0.4);
      border: 1px solid rgba(255,255,255,0.1);
      padding: 16px;
      max-width: 460px;
    ">
      <div style="
        font-size: 15px;
        font-weight: 600;
        margin-bottom: 12px;
        color: #f59e0b;
      ">
        Confirm Command
      </div>

      <div style="
        background: #0d1117;
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 6px;
        padding: 10px 12px;
        font-family: 'SF Mono', 'Fira Code', monospace;
        font-size: 13px;
        color: #7dd3fc;
        margin-bottom: 12px;
        word-break: break-all;
        user-select: text;
        -webkit-user-select: text;
      " id="command-text">
        ${escapeHtml(payload.command)}
      </div>

      <div style="
        font-size: 13px;
        color: rgba(255,255,255,0.7);
        margin-bottom: 16px;
        line-height: 1.5;
      ">
        ${escapeHtml(payload.explanation)}
      </div>

      <div id="result-area" style="display:none;margin-bottom:12px;"></div>

      <div style="display:flex;gap:8px;justify-content:flex-end;" id="button-row">
        <button id="btn-cancel" style="
          background: transparent;
          border: 1px solid rgba(255,255,255,0.2);
          color: rgba(255,255,255,0.8);
          padding: 6px 16px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 13px;
        ">Cancel</button>
        <button id="btn-run" style="
          background: #16a34a;
          border: none;
          color: white;
          padding: 6px 16px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 13px;
          font-weight: 500;
        ">Run</button>
      </div>
    </div>
  `;

  document.getElementById("btn-cancel")!.addEventListener("click", async () => {
    try {
      await invoke("close_action_menu");
    } catch { /* closing */ }
    window.close();
  });

  document.getElementById("btn-run")!.addEventListener("click", async () => {
    const runBtn = document.getElementById("btn-run") as HTMLButtonElement;
    const resultArea = document.getElementById("result-area")!;
    runBtn.disabled = true;
    runBtn.textContent = "Running...";
    runBtn.style.opacity = "0.6";

    try {
      const output = await invoke<string>("run_confirmed_command", {
        command: payload.command,
      });

      resultArea.style.display = "block";
      resultArea.innerHTML = `
        <div style="
          background: #052e16;
          border: 1px solid #16a34a;
          border-radius: 6px;
          padding: 8px 12px;
          font-size: 12px;
          color: #4ade80;
          font-family: monospace;
          max-height: 100px;
          overflow-y: auto;
          white-space: pre-wrap;
        ">${escapeHtml(output || "Command completed successfully.")}</div>
      `;

      runBtn.textContent = "Done";
      setTimeout(() => window.close(), 2000);
    } catch (err) {
      resultArea.style.display = "block";
      resultArea.innerHTML = `
        <div style="
          background: #450a0a;
          border: 1px solid #dc2626;
          border-radius: 6px;
          padding: 8px 12px;
          font-size: 12px;
          color: #fca5a5;
          font-family: monospace;
          max-height: 100px;
          overflow-y: auto;
          white-space: pre-wrap;
        ">${escapeHtml(String(err))}</div>
      `;

      runBtn.textContent = "Failed";
      runBtn.style.background = "#dc2626";
    }
  });
}

// Escape key closes the dialog
document.addEventListener("keydown", (e: KeyboardEvent) => {
  if (e.key === "Escape") {
    window.close();
  }
});

// Listen for the confirmation payload from the action menu
async function init(): Promise<void> {
  listen<ConfirmPayload>("confirm-command", (event) => {
    renderDialog(event.payload);
  });
}

init();
