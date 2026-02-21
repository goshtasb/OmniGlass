# Omni-Glass

**AI can read your screen. Omni-Glass lets it act on what it sees.**

Snip any region of your screen — a terminal error, a data table, a foreign-language doc — and Omni-Glass doesn't just explain it. It fixes the error. Exports the table. Translates the text. Creates the Jira ticket. Runs the command. All through sandboxed plugins that anyone can build.

It's an open-source execution layer between your screen and your tools.

![Omni-Glass Demo](docs/assets/demo.gif)

## See it work

**Snip a Python traceback →** Omni-Glass reads the error, generates `pip install pandas`, and asks you to confirm. One click and it's fixed.

**Snip a data table →** A native save dialog opens. Your CSV is ready.

**Snip a bug report in Slack →** A GitHub issue is created in your repo with the context already filled in.

**Snip Japanese documentation →** The English translation is on your clipboard.

**Don't want to snip?** Type a command in plain English from the menu bar. Same engine, no screenshot needed.

## Why not just use Claude / ChatGPT?

Chatbots *tell* you what to do. You still have to do it yourself.

Omni-Glass *does* it. It connects the LLM to real tools — your GitHub, your terminal, your clipboard, your file system — through [MCP plugins](https://modelcontextprotocol.io/) that execute actions directly. And every plugin runs inside a kernel-level sandbox, so it can't touch anything you haven't approved.

| | Chatbot | Omni-Glass |
|---|---------|------------|
| Input | Copy-paste or screenshot + describe what you want | Draw a box. That's it. |
| Output | Text you read and act on manually | Executed action: file saved, issue created, command run |
| Extensibility | None | MCP plugins — build your own in 100 lines |
| Security | Full system access or cloud-only | Sandboxed. Home directory walled off at the kernel level. |
| Offline | No | Yes. Local OCR + local LLM. Nothing leaves your machine. |

## How it works

1. Draw a box on your screen
2. OCR runs on your device — Apple Vision on macOS, Windows OCR on Windows. **No screenshots are sent anywhere.**
3. The extracted text goes to an LLM — Claude, Gemini, or Qwen-2.5 running locally via llama.cpp
4. A menu of contextual actions appears in under 1 second
5. Click one. It executes.

## Built-in actions

| Action | What it does |
|--------|-------------|
| Explain Error | Explains what went wrong and why |
| Fix Error | Generates a shell command or corrected code — you confirm before it runs |
| Export CSV | Extracts tabular data into a CSV with a native save dialog |
| Explain This | Plain-English explanation of whatever you snipped |
| Copy Text | Copies OCR-extracted text to clipboard |
| Search Web | Opens a browser search for the snipped content |
| Quick Translate | Translates snipped text to your preferred language |

## Build your own actions

Omni-Glass is a platform. Any [MCP server](https://modelcontextprotocol.io/) over `stdio` can add actions to the menu. The LLM automatically translates raw screen text into the structured JSON your tool expects — you don't write prompt logic, just the API call.

**What you could build:**

- **Snip a Slack message →** create a Linear, Jira, or Asana ticket with context filled in
- **Snip a design mockup →** generate the Tailwind CSS that matches it
- **Snip a SQL error →** query your database schema and suggest the fix
- **Snip a log file →** send it to Datadog or Grafana as a tagged event
- **Snip a code snippet →** run it in a sandbox and return the output
- **Snip a receipt →** extract the total and log it to your expense tracker
- **Snip an API response →** generate the TypeScript types automatically
- **Snip a meeting invite →** check your Google Calendar for conflicts

Each of these is a single MCP server with one tool. Most are under 100 lines of code.

```bash
# Get started in 5 minutes
git clone https://github.com/goshtasb/omni-glass-plugin-template.git
cd omni-glass-plugin-template
# Edit index.js — add your tool, write your API call
# Copy to the plugins directory, restart Omni-Glass
```

Read the [Plugin Developer Guide](docs/plugin-guide.md) for the full walkthrough.

## Security

Every plugin runs inside a kernel-level macOS sandbox (`sandbox-exec`).

- **Your home directory is walled off.** Plugins cannot read anything under `/Users/` unless you explicitly approve a specific path.
- **API keys are stripped.** Your `ANTHROPIC_API_KEY`, `AWS_SECRET_ACCESS_KEY`, and other secrets are invisible to plugin processes.
- **Commands require confirmation.** You see every shell command before it runs.
- **PII is redacted.** Credit card numbers, SSNs, and API keys are scrubbed before text is sent to a cloud LLM.

When you install a plugin, a permission dialog shows exactly what it can access. You approve or deny.

## Bring your own key

There are no Omni-Glass servers. Your API key talks directly to the provider. We never see your data.

| Provider | Type | Speed |
|----------|------|-------|
| Claude Haiku | Cloud | ~3s full pipeline |
| Gemini Flash | Cloud | Built, not yet benchmarked |
| Qwen-2.5-3B | Local via llama.cpp | ~8-15s, fully offline |

Local mode means zero cloud dependency. Your screen content never leaves your machine.

## Quick start

Requires: macOS 12+, Rust, Node.js 18+

```bash
git clone https://github.com/goshtasb/omni-glass.git
cd omni-glass
npm install
npm run tauri dev
```

1. Click the Omni-Glass icon in your menu bar
2. Settings → paste your API key, or download a local model for offline use
3. Click "Snip Screen" → draw a box → see the action menu

## Project status

| Feature | Status |
|---------|--------|
| macOS snip → OCR → action menu | ✅ Working |
| 7 built-in actions | ✅ Working |
| MCP plugin system with sandbox | ✅ Working |
| Text launcher (type commands) | ✅ Working |
| Local LLM (Qwen-2.5 via llama.cpp) | ✅ Built, testing |
| GitHub Issues plugin | ✅ Working |
| Windows support | 🔧 Code written, untested on hardware |
| Linux support | 📋 Planned |
| Plugin registry (in-app install) | 📋 Planned |
| UI element detection (click buttons, fill forms) | 📋 Planned |

## Contributing

- **Break the sandbox.** If you can read `~/.ssh/id_rsa` from a plugin, that's a critical bug.
- **Test on Windows.** The code compiles but hasn't run on real hardware.
- **Port to Linux.** Tesseract OCR, Bubblewrap sandbox, Wayland tray support.
- **Build a plugin.** If it's useful, others will want it too.

## License

MIT
