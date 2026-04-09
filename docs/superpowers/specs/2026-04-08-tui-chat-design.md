# ZStar TUI Chat Interface Design Spec

## Overview

Replace the raw `println!`-based chat REPL with a full Ratatui TUI that renders agent interactions as color-coded cards with Markdown output, collapsible thinking/tool steps, and a Ghostty-style theme system.

## Visual Style

Card-based GitHub Dark aesthetic:
- **Purple-bordered cards** for thinking steps (collapsed by default: spinner + duration)
- **Green-bordered cards** for tool calls (summary visible, press `d` to expand args + result)
- **Full Markdown rendering** for LLM responses (bold, italic, code blocks with syntax highlighting, tables, lists)
- **Clear input/response boundary** — bold prompt line, separator between user input and agent output

## Architecture

### New module: `crates/app-host/src/tui/`

```
tui/
├── mod.rs           — pub mod exports, TuiChat entry point
├── app.rs           — ChatApp struct, Ratatui event loop
├── event.rs         — terminal event handling (crossterm → AppEvent)
├── widgets/
│   ├── mod.rs
│   ├── input_bar.rs     — user input with history
│   ├── response_list.rs — scrollable list of response blocks
│   ├── thinking_card.rs — collapsed thinking indicator
│   ├── tool_card.rs     — tool call card with expand/collapse
│   ├── markdown.rs      — Markdown rendering widget
│   └── prompt.rs        — user prompt display (input boundary)
├── theme.rs         — Theme struct, built-in themes, TOML loading
└── markdown/
    ├── mod.rs       — comrak AST → styled lines
    └── syntax.rs    — syntect highlighter for code blocks
```

### Event flow

```
crossterm events ──→ event::read() ──→ AppEvent::Key/Input
                                           ↓
tokio::mpsc ← LiveRunEvent ← agent loop ──→ AppEvent::Agent(LiveRunEvent)
                                           ↓
                                    ChatApp::update()
                                           ↓
                                    Ratatui::draw() → terminal
```

- `ChatApp` owns a `tokio::mpsc::Receiver<LiveRunEvent>` and a `crossterm::EventStream`
- The main loop uses `tokio::select!` to handle both terminal input and agent events concurrently
- Each `LiveRunEvent` updates the app state (adds cards, updates markdown, etc.)
- Ratatui redraws on every state change

### Key types

```rust
struct ChatApp {
    responses: Vec<ResponseBlock>,  // scrollable content
    input: InputState,              // current user input + history
    scroll_offset: u16,
    theme: Theme,
    agent_rx: mpsc::Receiver<LiveRunEvent>,
    active_tool: Option<ActiveTool>, // tool in progress
    thinking_active: bool,
    thinking_start: Instant,
}

enum ResponseBlock {
    UserPrompt { text: String },
    ThinkingCard { duration: Duration, content: Option<String>, collapsed: bool },
    ToolCard { name: String, args_summary: String, result: Option<String>, duration: Duration, expanded: bool },
    MarkdownContent { parsed: Vec<StyledLine> },
}
```

### Markdown rendering

- `comrak` parses Markdown to AST
- Custom walker converts AST nodes to `Vec<StyledLine>` (line of `(Style, String)` segments)
- Code blocks highlighted by `syntect` with the current theme's syntax set
- The `Markdown` widget renders `StyledLine` using Ratatui's `Span`/`Line` types
- Supports: bold, italic, strikethrough, inline code, code blocks, links, lists (ordered/unordered), tables, headings (h1-h4)

### Thinking card behavior

- On `AgentEvent::MessageStarted`: show spinner + "Thinking..." label, start timer
- On next event (tool call, delta, completed): collapse the thinking card, show duration
- Thought text (reasoning_content if provider returns it) stored but hidden
- User can expand with `d` key if thought content is available

### Tool card behavior

- On `ToolCallReceived`: create card with tool name + args summary
- On `ToolExecutionCompleted`: fill result summary, show duration
- Press `d` to toggle expanded state (shows full args JSON + full result)
- Card border color: green for success, red for error, yellow for partial

### Input bar

- Bottom of screen, fixed position
- Shows `❯ ` prompt in theme accent color
- Supports: typing, backspace, left/right cursor, home/end, ctrl+w (delete word), up/down (history)
- Enter sends, ctrl+c exits, escape cancels current input
- In-chat commands: `/quit`, `/clear`, `/help`

### Scrolling

- Auto-scrolls to bottom as new content arrives
- User can scroll up with mouse wheel or pgup/pgdn
- Any new content while scrolled up shows a "↓ New content" indicator
- Pressing any key or scrolling to bottom resumes auto-scroll

## Theme System (Ghostty-style)

### Config location

- `~/.zstar/config/tui.toml` — user's TUI config (theme selection + keybindings)
- `~/.zstar/themes/` — custom theme TOML files
- Theme is only configurable via config files — no CLI flag or in-chat command

### Config structure

```toml
# ~/.zstar/config/tui.toml
theme = "github-dark"

[keybindings]
expand_detail = "d"          # key to expand/collapse tool and thinking cards
scroll_up = "pgup"
scroll_down = "pgdn"
```

### Theme file structure

```toml
# ~/./.zstar/themes/my-theme.toml
[name]
my-theme

[color]
background = "#0d1117"
foreground = "#c9d1d9"
border = "#30363d"
accent = "#58a6ff"

[color.card]
thinking_bg = "#161b22"
thinking_border = "#d2a8ff"
thinking_fg = "#8b949e"
tool_bg = "#161b22"
tool_border = "#238636"
tool_fg = "#8b949e"
tool_error_border = "#f85149"
error_fg = "#f85149"
success_fg = "#3fb950"

[color.syntax]
# syntect theme name or .tmTheme path
# options: "base16-ocean.dark", "base16-eighties.dark", "inspired-github", etc.
highlight_theme = "base16-eighties.dark"

[color.markdown]
heading = "#58a6ff"
bold = "#c9d1d9"
italic = "#8b949e"
code_bg = "#1c2128"
code_fg = "#79c0ff"
link = "#58a6ff"
```

### Built-in themes

| Name | Description |
|------|-------------|
| `github-dark` | GitHub Dark default. Dark bg, green tool borders, purple thinking. |
| `tokyo-night` | Tokyo Night Storm. Deeper blues, muted accents. |
| `light` | Clean light mode. White bg, blue accents, dark text. |

Built-in themes are embedded as `include_str!` TOML files. User themes override by name.

### Theme loading order

1. Check `tui.toml` `theme` field
2. Check for built-in with that name
3. Check `~/.zstar/themes/<name>.toml`
4. Fall back to `github-dark`

## Dependencies to add

```toml
# crates/app-host/Cargo.toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
comrak = "0.36"
syntect = "5.2"
toml = "0.8"
```

## Integration

- `zstar chat` uses the TUI interface
- The old `chat.rs` is replaced entirely by the new TUI module
- The TUI lives in `tui/` module and is called from `lib.rs`

## Testing

- Unit tests for theme loading, Markdown parsing, widget rendering
- Integration test: pipe events through the app state and verify response blocks
- Manual smoke test with `zstar chat`

## Success criteria

1. Thinking steps show as collapsed cards with spinner + duration
2. Tool calls show as expandable cards with name, args, result, timing
3. LLM responses render as full Markdown with syntax-highlighted code blocks
4. Clear visual boundary between user input and agent response
5. Theme configurable via config files only (`tui.toml` + `themes/` directory)
6. Scrolling works with auto-scroll and manual override
7. All existing functionality preserved (nudge, routing, subagents, hooks)
