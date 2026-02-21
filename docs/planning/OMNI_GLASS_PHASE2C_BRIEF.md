# Phase 2C: Ecosystem + Text Launcher

**Branch:** `feat/phase-2c-ecosystem`  
**Baseline:** `v0.3.0-phase2b` — MCP client working, sandbox enforced, 66/66 tests passing, zero performance regression  
**Goal:** Three first-party plugins ship. The text launcher works. A developer can build and load a custom plugin using our template and CLI in under 10 minutes.  
**Timeline:** Weeks 7-10 (3-4 weeks)

---

## What Phase 2C Changes

Phase 2A opened the platform boundary. Phase 2B secured it. Phase 2C populates it.

After Phase 2C, Omni-Glass is no longer a tool with a plugin system nobody uses. It's a tool with plugins that are useful from day one, a text input mode that doubles the product's surface area, and a developer experience that makes building plugins trivially easy. This is the phase that determines whether the ecosystem grows or dies.

The three deliverables are independent and can be built in parallel:

- **Track A: First-Party Plugins** — prove the platform works by building real, useful plugins on it
- **Track B: Text Launcher** — add a second input mode so users aren't limited to screenshots
- **Track C: Developer Experience** — template repo, CLI tool, documentation

---

## What Already Exists

| Component | Status | Phase 2C Role |
|-----------|--------|---------------|
| MCP client (stdio, JSON-RPC) | Working | First-party plugins connect through it |
| ToolRegistry + CLASSIFY injection | Working | Plugin tools appear in action menu automatically |
| EXECUTE pipeline with plugin dispatch | Working | Plugin tool calls route correctly |
| Sandbox (Broad System Allowlist) | Working | All plugins run sandboxed from day one |
| Permission prompt + approval state | Working | Users approve plugin permissions on first load |
| Test plugin (echo_text) | Working | Reference for plugin developers — but too simple to be useful |
| Settings panel | Working | Plugin management section needed |

---

## Track A: First-Party Plugins (Days 1-10)

### Why First-Party Plugins Matter

Nobody installs a plugin system with zero plugins. The first-party plugins serve three purposes:

1. **Immediate value** — users get useful functionality without installing anything extra
2. **Integration test** — if our own plugins work cleanly through the MCP pipeline, community plugins will too
3. **Reference implementations** — developers copy our plugin code as a starting point

### Plugin 1: GitHub Issues (Days 1-4)

**ID:** `com.omni-glass.github-issues`  
**Runtime:** Node.js  
**Why this plugin first:** It has the simplest permission model (network + one env var) and the broadest audience. Every developer uses GitHub.

**Manifest:**

```json
{
  "id": "com.omni-glass.github-issues",
  "name": "GitHub Issues",
  "version": "1.0.0",
  "description": "Create GitHub issues from snipped screen content",
  "author": {
    "name": "Omni-Glass Team",
    "url": "https://github.com/goshtasb/omni-glass"
  },
  "license": "MIT",
  "runtime": "node",
  "entry": "index.js",
  "minOmniGlassVersion": "0.3.0",
  "permissions": {
    "network": ["api.github.com"],
    "environment": ["GITHUB_TOKEN"],
    "clipboard": true
  },
  "configuration": {
    "default_repo": {
      "type": "string",
      "label": "Default Repository",
      "placeholder": "owner/repo",
      "description": "Default repository for new issues (e.g., goshtasb/omni-glass)"
    },
    "default_labels": {
      "type": "string",
      "label": "Default Labels",
      "placeholder": "bug,triage",
      "description": "Comma-separated labels to apply to new issues"
    }
  },
  "tools": [
    {
      "name": "create_github_issue",
      "description": "Create a GitHub issue from snipped content. Use when the user snips a bug, error, or feature request and wants to file it as an issue.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "title": { "type": "string", "description": "Issue title — concise summary of the problem" },
          "body": { "type": "string", "description": "Issue body — full description including any error text or context from the snip" },
          "repo": { "type": "string", "description": "Repository in owner/repo format. Uses configured default if omitted." },
          "labels": { "type": "array", "items": { "type": "string" }, "description": "Labels to apply. Uses configured defaults if omitted." }
        },
        "required": ["title", "body"]
      }
    }
  ]
}
```

**Implementation (`index.js`):**

The plugin is a standard MCP server over stdio. It:

1. Reads NDJSON from stdin
2. Responds to `initialize` with server info
3. Responds to `tools/list` with the `create_github_issue` tool
4. On `tools/call` for `create_github_issue`:
   - Reads `GITHUB_TOKEN` from environment
   - Reads `default_repo` and `default_labels` from plugin configuration (passed via initialize params or environment)
   - POSTs to `https://api.github.com/repos/{owner}/{repo}/issues` with title, body, and labels
   - Returns success with the issue URL, or error with the GitHub API error message

**Expected user flow:**

1. User snips a Python error in their terminal
2. Action menu shows: Explain Error, Fix Error, Copy Text, **Create GitHub Issue**
3. User clicks "Create GitHub Issue"
4. EXECUTE pipeline calls the LLM: "Given this error text, generate a title and body for a GitHub issue"
5. LLM returns structured JSON with title and body
6. Pipeline routes the tool call to the GitHub Issues plugin
7. Plugin creates the issue via GitHub API
8. Result displays: "✅ Created issue #42: ModuleNotFoundError in analysis.py — https://github.com/owner/repo/issues/42"

**Key implementation detail — the LLM bridge:**

The plugin's `create_github_issue` tool expects `title` and `body` as inputs. But the user clicked an action button — they didn't write a title. The EXECUTE pipeline needs to ask the LLM to generate the title and body from the OCR text, then pass those as arguments to the plugin's tool.

This means the EXECUTE flow for plugin tools is:

```
User clicks plugin action
    → EXECUTE LLM call: "Generate arguments for this tool given the snipped text"
    → LLM returns: { title: "...", body: "..." }
    → Pipeline calls plugin: tools/call("create_github_issue", { title, body })
    → Plugin hits GitHub API
    → Result returned to user
```

Add this LLM-to-tool-args bridge in `pipeline.rs`. The EXECUTE prompt needs a new mode: "Given this OCR text and this tool's input schema, generate the appropriate arguments as JSON."

**New prompt template in `prompts_execute.rs`:**

```
You are generating arguments for a plugin tool. Given the user's snipped screen
content and the tool's input schema, produce a JSON object with the required fields.

Tool: {tool_name}
Description: {tool_description}
Input Schema: {input_schema_json}

Snipped Content:
<extracted_text>
{ocr_text}
</extracted_text>

Respond with ONLY a valid JSON object matching the input schema. No explanation.
```

**Definition of done:**

- [ ] Plugin creates a real GitHub issue when `GITHUB_TOKEN` is set
- [ ] Plugin loads through the standard MCP pipeline (manifest → approval → sandbox → discover)
- [ ] "Create GitHub Issue" appears in the action menu when snipping error/bug content
- [ ] Result shows the issue URL
- [ ] Without `GITHUB_TOKEN`, the plugin loads but the tool call returns a clear error ("GITHUB_TOKEN not configured")
- [ ] Plugin runs inside the sandbox — cannot read `~/.ssh` or any undeclared paths

### Plugin 2: Quick Translate (Days 3-6)

**ID:** `com.omni-glass.quick-translate`  
**Runtime:** Node.js  
**Why this plugin:** It demonstrates a different pattern — the plugin doesn't call an external API, it delegates back to the user's configured LLM. It's also universally useful (snip Japanese docs → get English translation).

**Manifest:**

```json
{
  "id": "com.omni-glass.quick-translate",
  "name": "Quick Translate",
  "version": "1.0.0",
  "description": "Translate snipped text to your preferred language",
  "author": {
    "name": "Omni-Glass Team",
    "url": "https://github.com/goshtasb/omni-glass"
  },
  "license": "MIT",
  "runtime": "node",
  "entry": "index.js",
  "minOmniGlassVersion": "0.3.0",
  "permissions": {
    "clipboard": true
  },
  "configuration": {
    "target_language": {
      "type": "string",
      "label": "Target Language",
      "placeholder": "English",
      "description": "Language to translate snipped text into"
    }
  },
  "tools": [
    {
      "name": "translate_text",
      "description": "Translate the snipped text to the user's preferred language. Use when the user snips text in a foreign language.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "text": { "type": "string", "description": "The text to translate" },
          "target_language": { "type": "string", "description": "Target language for translation" }
        },
        "required": ["text", "target_language"]
      }
    }
  ]
}
```

**Implementation pattern — LLM-delegating plugin:**

This plugin doesn't make any network calls itself. It receives the text and target language as tool arguments, and its result is simply the translation request formatted for the host application. The actual translation happens in the EXECUTE pipeline's LLM call that generates the tool arguments — the LLM is asked to translate the text as part of generating the `text` argument.

Wait — that's circular. Let me think about this differently.

**Better pattern: The plugin returns an instruction, not a result.**

```
tools/call("translate_text", { text: "日本語のテキスト", target_language: "English" })
    → Plugin returns: { type: "llm_request", prompt: "Translate the following to English: 日本語のテキスト" }
    → Pipeline sees type "llm_request" → sends to LLM → returns translation to user
```

Actually, the cleanest pattern: **the plugin is just the trigger.** The EXECUTE pipeline's LLM call already has the OCR text. The tool's description tells the LLM to translate. The plugin's `tools/call` handler just returns the translation directly — because the LLM already generated it as part of the tool arguments step.

**Simplest correct implementation:**

1. LLM sees "translate_text" tool with `text` and `target_language` params
2. EXECUTE prompt generates tool args: `{ text: "<translated text>", target_language: "English" }`
3. But wait — the LLM is generating the *arguments*, not the *result*. The `text` field is the input to translate, not the output.

**Correct architecture:**

The plugin itself performs the translation. But it has no network access (no API key). Two options:

**Option A: Plugin calls back to the host LLM.** The MCP spec supports "sampling" — the server asks the client to run an LLM completion. This is the MCP-native way but adds complexity.

**Option B: Plugin is a thin shell. The EXECUTE pipeline does the translation.**

Use Option B for v1. The plugin's tool handler receives `{ text, target_language }`, constructs a prompt ("Translate this to {target_language}: {text}"), and returns it as a `{ type: "text", text: "<prompt>" }` result that the pipeline knows to process via the LLM. Alternatively, the pipeline detects that the plugin is requesting LLM processing and handles it.

**Simplest correct approach for v1:**

Skip the MCP round-trip entirely. Register `translate_text` as a built-in tool (like `explain_text`) that uses the EXECUTE LLM call with a translation-specific prompt. The plugin manifest exists to declare the action so it appears in the CLASSIFY menu, but execution is handled internally.

This is pragmatic: the first-party translate plugin is effectively a "smart alias" that teaches the CLASSIFY LLM to offer translation as an action. The actual work happens in the EXECUTE pipeline with a translation prompt.

```rust
// In prompts_execute.rs
pub const PROMPT_TRANSLATE: &str = r#"
Translate the following text to {target_language}.
Return ONLY the translation, with no explanation or commentary.
Preserve the original formatting (paragraphs, line breaks, lists).

Text to translate:
{extracted_text}
"#;
```

**Definition of done:**

- [ ] "Translate to {language}" appears in the action menu when snipping foreign-language text
- [ ] Clicking it produces a translation displayed in the action menu popup
- [ ] Translation is automatically copied to clipboard
- [ ] Target language is configurable in Settings → Plugins → Quick Translate
- [ ] Works with the `.accurate` OCR mode for higher fidelity on non-Latin scripts

### Plugin 3: ScreenPipe Bridge (Days 6-10)

**ID:** `com.omni-glass.screenpipe-bridge`  
**Runtime:** Node.js  
**Why this plugin:** It demonstrates the most powerful plugin pattern — connecting Omni-Glass to an external local service. It also creates a partnership opportunity with the ScreenPipe project.

**Manifest:**

```json
{
  "id": "com.omni-glass.screenpipe-bridge",
  "name": "ScreenPipe Bridge",
  "version": "1.0.0",
  "description": "Search your screen history using ScreenPipe's local database",
  "author": {
    "name": "Omni-Glass Team",
    "url": "https://github.com/goshtasb/omni-glass"
  },
  "license": "MIT",
  "runtime": "node",
  "entry": "index.js",
  "minOmniGlassVersion": "0.3.0",
  "permissions": {
    "network": ["localhost:3030"]
  },
  "tools": [
    {
      "name": "screenpipe_search",
      "description": "Search the user's screen recording history for past occurrences of snipped content. Use when the user wants to find when they previously saw something on their screen.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string", "description": "Search query — text to find in screen history" },
          "minutes_ago": { "type": "integer", "description": "How far back to search, in minutes. Default: 1440 (24 hours)" },
          "content_type": { "type": "string", "enum": ["ocr", "audio", "all"], "description": "Type of content to search. Default: ocr" }
        },
        "required": ["query"]
      }
    }
  ]
}
```

**Implementation:**

```javascript
// On tools/call("screenpipe_search", { query, minutes_ago, content_type })
const response = await fetch(`http://localhost:3030/search`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    q: args.query,
    content_type: args.content_type || "ocr",
    limit: 10,
    start_time: new Date(Date.now() - (args.minutes_ago || 1440) * 60000).toISOString(),
    end_time: new Date().toISOString()
  })
});
```

**Dependency:** User must have ScreenPipe installed and running on port 3030. If ScreenPipe isn't running, the plugin should return a clear error: "ScreenPipe is not running. Install it from https://screenpi.pe and start it before using this plugin."

**Expected user flow:**

1. User snips an invoice number on screen
2. Action menu shows: Copy Text, Explain, **Find in History**
3. User clicks "Find in History"
4. Plugin queries ScreenPipe: "When did I last see this invoice?"
5. Result: "Found 3 matches in the last 24 hours: 2:15 PM (Chrome), 11:30 AM (Mail), yesterday 4:00 PM (Slack)"

**Definition of done:**

- [ ] Plugin connects to ScreenPipe's local API on localhost:3030
- [ ] "Find in History" appears in the action menu
- [ ] Search results display in the action menu popup
- [ ] Clear error message when ScreenPipe isn't running
- [ ] Plugin's network access restricted to localhost:3030 only (sandbox enforced)

---

## Track B: Text Launcher (Days 3-6)

### What It Is

A Spotlight-style text input bar triggered by a global hotkey. The user types a natural language command, and it goes directly to the EXECUTE pipeline — skipping the snip and CLASSIFY steps entirely.

### How It Works

| Step | What Happens |
|------|-------------|
| 1. Trigger | User presses `Cmd+Shift+Space` (macOS) or `Ctrl+Shift+Space` (Windows) |
| 2. Window | A centered, single-line text input appears. Dark theme, matches action menu aesthetic. |
| 3. Type | User types a command: "Draft a Jira ticket for the broken login button" |
| 4. Submit | User presses Enter |
| 5. Execute | Text goes to EXECUTE pipeline with all installed plugin tools available |
| 6. Result | Result renders in the same action menu popup used for snip results |
| 7. Dismiss | Escape or click outside closes the launcher |

### Why Skip CLASSIFY

CLASSIFY exists to analyze visual content and determine what actions make sense. With text input, the user is explicitly stating their intent — there's nothing to classify. The text goes directly to EXECUTE with the full set of available tools (built-in + plugin).

### Implementation

**New files:**

```
src/text-launcher.html    (~40 lines)  — Minimal HTML: input field + styles
src/text-launcher.ts      (~80 lines)  — Event handlers: Enter to submit, Escape to close
```

**Modified files:**

```
src-tauri/src/lib.rs          — Register global hotkey, create launcher window
src-tauri/src/commands.rs      — New command: execute_text_command(text: String)
src-tauri/src/pipeline.rs      — New function: execute_text_input() — builds EXECUTE prompt with all tools
src-tauri/src/llm/prompts_execute.rs  — New prompt: PROMPT_TEXT_COMMAND
```

**Global hotkey registration:**

```rust
// In lib.rs .setup() hook
use tauri_plugin_global_shortcut::ShortcutState;

app.plugin(
    tauri_plugin_global_shortcut::Builder::new()
        .with_shortcut("CmdOrCtrl+Shift+Space")?
        .with_handler(|app, shortcut, event| {
            if event.state == ShortcutState::Pressed {
                // Create or show the text launcher window
                show_text_launcher(app);
            }
        })
        .build()
)?;
```

**Text launcher window:**

```rust
fn show_text_launcher(app: &AppHandle) {
    // Get the focused monitor's dimensions for centering
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "text-launcher",
        tauri::WebviewUrl::App("text-launcher.html".into()),
    )
    .title("")
    .inner_size(600.0, 52.0)        // Thin bar, 600px wide
    .center()                        // Center on screen
    .decorations(false)              // No title bar
    .always_on_top(true)
    .transparent(true)
    .focused(true)
    .skip_taskbar(true)
    .build()
    .unwrap();
}
```

**Text launcher HTML:**

```html
<div id="launcher">
  <input
    type="text"
    id="input"
    placeholder="Type a command... (plugins: GitHub Issues, Translate, ScreenPipe)"
    autofocus
  />
</div>
```

**Styling:** Dark background matching the action menu (#1a1a2e), white text, no border, rounded corners, subtle shadow. The input fills the entire width. The placeholder text lists installed plugin names so the user knows what's available.

**Text launcher TypeScript:**

```typescript
const input = document.getElementById("input") as HTMLInputElement;

input.addEventListener("keydown", async (e) => {
  if (e.key === "Enter" && input.value.trim()) {
    // Send text to EXECUTE pipeline
    const result = await invoke("execute_text_command", {
      text: input.value.trim()
    });
    // Close launcher, show result in action menu popup
    await invoke("show_action_result", { result });
    await invoke("close_text_launcher");
  }
  if (e.key === "Escape") {
    await invoke("close_text_launcher");
  }
});

// Click outside to close
window.addEventListener("blur", async () => {
  await invoke("close_text_launcher");
});
```

**EXECUTE prompt for text commands:**

```rust
pub const PROMPT_TEXT_COMMAND: &str = r#"
You are Omni-Glass, a Visual Action Engine. The user typed a text command
(they did not snip the screen). Execute their request using the available tools.

Available tools:
{available_tools}

User command: {user_text}

If a tool matches the user's intent, respond with:
{
  "status": "success",
  "actionId": "<tool_name>",
  "result": { "type": "text", "text": "<your response>" },
  "toolCall": {
    "name": "<tool_name>",
    "arguments": { <tool arguments> }
  }
}

If no tool matches and you can answer directly, respond with:
{
  "status": "success",
  "actionId": "direct_response",
  "result": { "type": "text", "text": "<your helpful response>" }
}

If the user's request requires a tool that isn't installed, suggest which plugin
they might need.
"#;
```

**Pipeline integration (`pipeline.rs`):**

```rust
pub async fn execute_text_command(
    text: &str,
    registry: &ToolRegistry,
) -> Result<ActionResult, String> {
    // 1. Get all available tools (built-in + plugin)
    let tools_description = registry.tools_for_prompt();
    
    // 2. Build the EXECUTE prompt with all tools
    let prompt = PROMPT_TEXT_COMMAND
        .replace("{available_tools}", &tools_description)
        .replace("{user_text}", text);
    
    // 3. Call the LLM
    let response = llm_execute_call(&prompt).await?;
    
    // 4. If the response includes a toolCall, route it
    if let Some(tool_call) = response.tool_call {
        if registry.is_plugin_action(&tool_call.name) {
            return execute_plugin_tool(registry, &tool_call.name, &tool_call.arguments).await;
        }
    }
    
    // 5. Return the text result
    Ok(response)
}
```

**Definition of done:**

- [ ] `Cmd+Shift+Space` opens the text launcher
- [ ] Typing "Create a GitHub issue for the broken login button" → creates an issue (if plugin installed + configured)
- [ ] Typing "Translate 'hello world' to Japanese" → returns translation
- [ ] Typing "What is 2+2?" → LLM responds directly without a tool
- [ ] Escape closes the launcher
- [ ] Click outside closes the launcher
- [ ] Launcher shows installed plugin names in placeholder text

---

## Track C: Developer Experience (Days 5-10)

### C1: Plugin Template Repository (Days 5-7)

**Repository:** `github.com/goshtasb/omni-glass-plugin-template`

A GitHub template repository that a developer clones to start building a plugin. It must be dead simple — `git clone`, `npm install`, edit one file, and you have a working plugin.

**Template structure:**

```
omni-glass-plugin-template/
  omni-glass.plugin.json    — Manifest with placeholders
  index.js                   — MCP server boilerplate with one example tool
  package.json               — Dependencies: none (pure Node.js stdio)
  README.md                  — Step-by-step guide
  .gitignore
  LICENSE                    — MIT
```

**Template `index.js`:**

```javascript
#!/usr/bin/env node

/**
 * Omni-Glass Plugin Template
 * 
 * This is a minimal MCP server that communicates with Omni-Glass
 * over stdio using JSON-RPC 2.0.
 * 
 * To build your plugin:
 * 1. Edit omni-glass.plugin.json with your plugin's info and permissions
 * 2. Add your tools to the TOOLS array below
 * 3. Implement your tool handlers in handleToolCall()
 * 4. Test with: omni-glass plugin dev
 */

const readline = require("readline");

// ═══ Define your tools here ═══

const TOOLS = [
  {
    name: "my_tool",
    description: "Describe what your tool does — the LLM reads this to decide when to use it",
    inputSchema: {
      type: "object",
      properties: {
        text: { type: "string", description: "The snipped text or user input" }
      },
      required: ["text"]
    }
  }
];

// ═══ Implement your tool handlers here ═══

async function handleToolCall(name, args) {
  switch (name) {
    case "my_tool":
      // Your logic here
      return {
        content: [{ type: "text", text: `Received: ${args.text}` }]
      };
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
}

// ═══ MCP Server Boilerplate (don't edit below unless you know what you're doing) ═══

const rl = readline.createInterface({ input: process.stdin });
let nextId = 1;

function send(msg) {
  process.stdout.write(JSON.stringify(msg) + "\n");
}

rl.on("line", async (line) => {
  try {
    const msg = JSON.parse(line);
    
    if (msg.method === "initialize") {
      send({ jsonrpc: "2.0", id: msg.id, result: {
        protocolVersion: "2024-11-05",
        capabilities: { tools: {} },
        serverInfo: { name: "my-plugin", version: "0.1.0" }
      }});
    } else if (msg.method === "notifications/initialized") {
      // Ready
    } else if (msg.method === "tools/list") {
      send({ jsonrpc: "2.0", id: msg.id, result: { tools: TOOLS }});
    } else if (msg.method === "tools/call") {
      const result = await handleToolCall(msg.params.name, msg.params.arguments);
      send({ jsonrpc: "2.0", id: msg.id, result });
    }
  } catch (err) {
    send({ jsonrpc: "2.0", id: null, error: { code: -32603, message: err.message }});
  }
});
```

**Template README.md:**

```markdown
# Omni-Glass Plugin Template

Build a plugin for [Omni-Glass](https://github.com/goshtasb/omni-glass) in 5 minutes.

## Quick Start

1. Click "Use this template" on GitHub (or clone directly)
2. Edit `omni-glass.plugin.json` with your plugin's info
3. Edit `index.js` — add your tools and handlers
4. Install your plugin:
   ```bash
   cp -r . ~/.config/omni-glass/plugins/com.your-name.your-plugin/
   ```
   (On macOS: `~/Library/Application Support/omni-glass/plugins/`)
5. Restart Omni-Glass — your plugin loads automatically

## Testing

```bash
# Send a test tool call
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | node index.js
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | node index.js
```

## Permissions

Declare what your plugin needs in `omni-glass.plugin.json`:

| Permission | Example | What It Grants |
|-----------|---------|----------------|
| network | `["api.github.com"]` | HTTPS to listed domains |
| filesystem | `[{"path": "~/Documents", "access": "read"}]` | File access |
| environment | `["MY_API_KEY"]` | Read specific env vars |
| clipboard | `true` | Read/write clipboard |
| shell | `{"commands": ["git"]}` | Run specific commands |

Users approve permissions when they first install your plugin.

## Publishing

Share your plugin by publishing the folder to GitHub. Users install by cloning
into their plugins directory. A plugin registry is coming in a future release.
```

**Definition of done:**

- [ ] Repository exists at `github.com/goshtasb/omni-glass-plugin-template`
- [ ] "Use this template" creates a working plugin with zero modifications
- [ ] The example tool (`my_tool`) loads in Omni-Glass and appears in the action menu
- [ ] README is accurate and complete

### C2: CLI Tool (Days 7-9)

**Package:** `@omni-glass/cli` (npm)

A CLI tool for plugin development. It's a convenience layer — everything it does can be done manually, but the CLI makes it faster.

**Commands:**

| Command | What It Does | Implementation |
|---------|-------------|----------------|
| `omni-glass plugin init` | Scaffolds a new plugin from the template in the current directory | Downloads template from GitHub, replaces placeholders with user input |
| `omni-glass plugin dev` | Starts the plugin as an MCP server and connects it to a running Omni-Glass instance for live testing | Copies plugin to the plugins directory, restarts the MCP loader, tails the log |
| `omni-glass plugin test` | Sends sample tool calls and validates responses | Spawns the plugin, sends initialize + tools/list + tools/call, checks response format |
| `omni-glass plugin validate` | Checks the manifest for correctness | Validates manifest schema, permissions, tool definitions |
| `omni-glass plugin package` | Bundles the plugin into a distributable archive | Creates a .tar.gz with the plugin directory, verifies all declared files exist |

**Implementation: Pure Node.js, zero dependencies beyond built-in modules.**

The CLI is a single `bin/omni-glass.js` file (or split into subcommand files if it gets large). It uses `readline` for interactive prompts, `child_process` for spawning plugins, `fs` for file operations, and `https` for downloading the template.

**`omni-glass plugin init` interactive flow:**

```
$ omni-glass plugin init

  Omni-Glass Plugin Generator

  Plugin name: Slack Notifier
  Plugin ID (reverse-domain): com.myname.slack-notifier
  Description: Send snipped content to Slack channels
  Runtime (node/python): node
  
  Permissions needed:
    Network domains (comma-separated, or 'none'): hooks.slack.com
    Environment variables (comma-separated, or 'none'): SLACK_WEBHOOK_URL
    Clipboard access? (y/n): n
    Shell commands (comma-separated, or 'none'): none
  
  ✅ Created plugin in ./slack-notifier/
  
  Next steps:
    cd slack-notifier
    # Edit index.js to add your tool logic
    omni-glass plugin test     # Verify it works
    omni-glass plugin dev      # Live-test in Omni-Glass
```

**`omni-glass plugin test` output:**

```
$ omni-glass plugin test

  Testing plugin: com.myname.slack-notifier

  ✅ Manifest valid
  ✅ Server starts (initialize response in 124ms)
  ✅ Tools discovered: 1 tool (send_to_slack)
  ✅ Tool call succeeded (tools/call response in 87ms)
  ✅ Server shuts down cleanly

  All checks passed.
```

**Definition of done:**

- [ ] `npm install -g @omni-glass/cli` works
- [ ] `omni-glass plugin init` scaffolds a working plugin
- [ ] `omni-glass plugin test` validates a plugin without Omni-Glass running
- [ ] `omni-glass plugin validate` catches manifest errors
- [ ] `omni-glass plugin dev` installs the plugin into the running instance

### C3: Developer Documentation (Days 8-10)

**Location:** `docs/` directory in the main repo + GitHub Pages or README links

| Document | Contents | Priority |
|----------|----------|----------|
| `docs/plugin-guide.md` | Step-by-step: Build Your First Plugin. From zero to working plugin in 10 minutes. Includes the Jira example walkthrough. | P0 |
| `docs/mcp-api-reference.md` | Complete JSON-RPC message schema, tool definition format, error codes, lifecycle hooks. Every message the MCP client sends and expects. | P0 |
| `docs/manifest-reference.md` | Every field in `omni-glass.plugin.json` with examples, types, validation rules, and common mistakes. | P0 |
| `docs/security-model.md` | How the sandbox works, what permissions mean, what plugins can and cannot do, known limitations (e.g., denylist note). | P1 |
| `docs/architecture.md` | System architecture overview for contributors: Shell/Eyes/Brain/Hands layers, data flow, module structure. | P1 |

**Plugin Guide structure (plugin-guide.md):**

1. Prerequisites (Node.js 18+, Omni-Glass v0.3.0+)
2. Create a plugin (`omni-glass plugin init` or clone template)
3. Define your tools (manifest + tool schema)
4. Implement your handlers (index.js)
5. Test locally (`omni-glass plugin test`)
6. Install and try it (`omni-glass plugin dev`)
7. Declare permissions (what to request and why)
8. Handle errors gracefully
9. Distribute your plugin (GitHub, share the folder)
10. Example: Building a Weather plugin from scratch

**Definition of done:**

- [ ] Plugin guide is accurate — a developer can follow it from zero to working plugin
- [ ] API reference covers every JSON-RPC message
- [ ] Manifest reference covers every field with examples
- [ ] All docs are in the repo under `docs/`

---

## Settings Panel: Plugin Management (Days 4-6)

### New Section in Settings

The existing settings panel gets a "Plugins" section:

```
┌─────────────────────────────────────────────────────────┐
│  Plugins                                                 │
│  ─────────────────────────────────────────────────────   │
│                                                          │
│  ┌─ GitHub Issues ─────────────────────────────────┐    │
│  │  com.omni-glass.github-issues · v1.0.0           │    │
│  │  Status: ✅ Active (1 tool)                       │    │
│  │  Permissions: 🌐 api.github.com · 🔑 GITHUB_TOKEN│    │
│  │                                                    │    │
│  │  Configuration:                                    │    │
│  │  Default Repo: [goshtasb/omni-glass    ]          │    │
│  │  Default Labels: [bug,triage            ]          │    │
│  │                                                    │    │
│  │  [Configure]  [Disable]  [Remove]                 │    │
│  └────────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─ Quick Translate ───────────────────────────────┐    │
│  │  com.omni-glass.quick-translate · v1.0.0         │    │
│  │  Status: ✅ Active (1 tool)                       │    │
│  │  Permissions: 📋 Clipboard                        │    │
│  │                                                    │    │
│  │  Configuration:                                    │    │
│  │  Target Language: [English              ]          │    │
│  │                                                    │    │
│  │  [Configure]  [Disable]  [Remove]                 │    │
│  └────────────────────────────────────────────────────┘   │
│                                                          │
│  Plugin Directory: ~/Library/Application Support/        │
│  omni-glass/plugins/                                     │
│                                                          │
│  [Open Plugins Folder]  [Refresh Plugins]                │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**"Open Plugins Folder"** opens the plugins directory in Finder/Explorer.  
**"Refresh Plugins"** re-scans the directory and loads any new plugins.  
**"Disable"** stops the plugin's MCP server and removes its tools from the registry. Plugin stays installed.  
**"Remove"** stops the server, removes approval, deletes the plugin directory.

**Modified files:**

```
src/settings.ts              — Add plugins section
src-tauri/src/settings_commands.rs  — New commands: list_plugins, disable_plugin, remove_plugin, update_plugin_config
```

---

## New File Structure (All Tracks)

```
First-party plugins (bundled with app):
  plugins/com.omni-glass.github-issues/
    omni-glass.plugin.json
    index.js
    package.json
  plugins/com.omni-glass.quick-translate/
    omni-glass.plugin.json
    index.js
  plugins/com.omni-glass.screenpipe-bridge/
    omni-glass.plugin.json
    index.js

Text launcher:
  src/text-launcher.html
  src/text-launcher.ts

Modified Rust:
  src-tauri/src/lib.rs                    — Global hotkey registration
  src-tauri/src/commands.rs               — execute_text_command
  src-tauri/src/pipeline.rs               — execute_text_input(), LLM-to-tool-args bridge
  src-tauri/src/llm/prompts_execute.rs    — PROMPT_TEXT_COMMAND, PROMPT_TRANSLATE, PROMPT_PLUGIN_ARGS
  src-tauri/src/settings_commands.rs      — Plugin management commands

CLI tool (separate npm package):
  cli/
    bin/omni-glass.js
    package.json
    README.md

Documentation:
  docs/
    plugin-guide.md
    mcp-api-reference.md
    manifest-reference.md
    security-model.md
    architecture.md
```

---

## Verification Checklist

### Track A: First-Party Plugins

- [ ] GitHub Issues plugin creates a real issue with `GITHUB_TOKEN` set
- [ ] Quick Translate translates snipped foreign text and copies to clipboard
- [ ] ScreenPipe Bridge returns search results when ScreenPipe is running
- [ ] All three plugins load through standard MCP pipeline (sandbox, approval, discovery)
- [ ] All three plugin tools appear in the action menu when relevant content is snipped
- [ ] Plugin configuration works (default repo, target language)
- [ ] Graceful errors when external dependencies are missing (no GITHUB_TOKEN, ScreenPipe not running)

### Track B: Text Launcher

- [ ] Cmd+Shift+Space opens the launcher
- [ ] Enter submits, Escape and click-outside close
- [ ] Text commands execute through the EXECUTE pipeline
- [ ] Plugin tools are available via text commands
- [ ] Direct LLM responses work for non-tool queries
- [ ] Placeholder shows installed plugin names

### Track C: Developer Experience

- [ ] Plugin template generates a working plugin
- [ ] CLI `init` scaffolds correctly
- [ ] CLI `test` validates a plugin without Omni-Glass running
- [ ] Plugin guide is followable by a developer with no prior context
- [ ] API reference covers all MCP messages

### Regression

- [ ] All 66 existing tests still pass
- [ ] Built-in actions work identically
- [ ] Sandbox enforcement unchanged
- [ ] Snip pipeline performance unchanged (< 3.5s full actions)

---

## What NOT to Build

| Don't | Why |
|-------|-----|
| Plugin registry / marketplace | Phase 3. Plugins install from local directory or GitHub for now. |
| Plugin update mechanism | Phase 3. Manual reinstallation for now. |
| Plugin signing / code verification | Phase 3. Trust is established via permission prompt + sandbox. |
| Rich plugin UI (custom webviews) | Phase 3. Plugins return text/file/command results only. |
| Multi-turn conversation | Phase 3. Each action is a single LLM call. |
| Local LLM / offline mode | Phase 3. Requires llama.cpp integration. |
| Linux / Windows sandbox hardening | Phase 3. Environment filtering is the baseline. |

---

## End-of-Phase-2C Gate

**The gate has three parts:**

1. **Plugin gate:** At least two of three first-party plugins are functional end-to-end (loaded, sandboxed, tools discoverable, tool calls execute, results display).

2. **Launcher gate:** Text launcher opens via hotkey, submits a command, and displays a result. Bonus if it routes to a plugin tool.

3. **Developer gate:** An external person (not on the team) follows the plugin guide and builds a custom plugin that loads in Omni-Glass. If they can do it in under 30 minutes, the developer experience is good enough.

When all three gates pass, Phase 2 is complete. Omni-Glass is a platform with an ecosystem. Tag `v0.4.0-phase2c` and ship it.
