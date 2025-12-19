# Blackbird Architecture

This document provides a comprehensive technical overview of Blackbird's architecture, design decisions, and implementation details.

---

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [System Architecture](#system-architecture)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [AI Integration](#ai-integration)
- [Bridge System](#bridge-system)
- [Storage Architecture](#storage-architecture)
- [Theming System](#theming-system)
- [Platform Support](#platform-support)

---

## Overview

Blackbird is built on a **reactive component architecture** using Rust and Dioxus. The application follows a unidirectional data flow pattern with signals for state management, enabling efficient UI updates and cross-platform compilation.

```
┌─────────────────────────────────────────────────────────────────┐
│                         Blackbird App                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Workbench  │  │    Apps     │  │        Settings         │  │
│  │    View     │  │   Gallery   │  │          View           │  │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘  │
│         │                │                      │               │
│         └────────────────┼──────────────────────┘               │
│                          │                                      │
│                    ┌─────┴─────┐                                │
│                    │  Signals  │  (Reactive State)              │
│                    └─────┬─────┘                                │
│                          │                                      │
├──────────────────────────┼──────────────────────────────────────┤
│  ┌───────────────────────┴───────────────────────────────────┐  │
│  │                    Service Layer                          │  │
│  ├─────────────────┬─────────────────┬──────────────────────┤  │
│  │   AI Client     │   Storage       │   Bridge (SDK)       │  │
│  │   (Streaming)   │   (Persistence) │   (App Runtime)      │  │
│  └─────────────────┴─────────────────┴──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

### Core Framework

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust 2024 Edition | Memory safety, performance, cross-platform |
| UI Framework | Dioxus 0.6.3 | React-like components, signals, multi-platform |
| Async Runtime | Tokio | Concurrent operations, streaming |

### AI Integration

| Component | Technology | Purpose |
|-----------|------------|---------|
| LLM Client | Rig 0.23 | Unified multi-provider AI interface |
| Streaming | Custom polling | Non-blocking response streaming |

### Utilities

| Component | Technology | Purpose |
|-----------|------------|---------|
| Markdown | Comrak | GFM-compatible markdown parsing |
| Syntax Highlighting | Syntect | Code block highlighting |
| HTTP | Reqwest | API communication |
| Serialization | Serde + JSON | Data persistence |
| Time | time crate | Timestamps, formatting |
| Clipboard | arboard | Copy-to-clipboard |

---

## System Architecture

### Module Structure

```
src/
├── main.rs           # Entry point, environment loading
├── ui.rs             # Root component, navigation, theming
├── types.rs          # Shared type definitions
├── theme.rs          # Theme definitions and CSS generation
├── bridge.rs         # Blackbird SDK injection system
│
├── views/
│   ├── mod.rs        # View exports
│   ├── workbench.rs  # AI app builder interface
│   ├── apps.rs       # Saved apps gallery
│   ├── chat.rs       # Document assistant (alt mode)
│   ├── settings.rs   # User preferences
│   └── shared.rs     # Shared utilities (persistence, markdown)
│
├── ai/
│   ├── mod.rs        # AI module exports
│   ├── client.rs     # BlackbirdAI unified client
│   └── providers/
│       ├── mod.rs    # Provider detection logic
│       └── blackbird.rs  # Custom API client
│
└── tools/
    ├── mod.rs        # Tool exports
    ├── calculator.rs # Math evaluation tool
    ├── apps.rs       # App search/list tools
    └── settings.rs   # Settings access tool
```

### Component Hierarchy

```
App (ui.rs)
├── Splash Screen
└── Main Layout
    ├── Tab Navigation
    └── Active View
        ├── Workbench
        │   ├── Prompt Input
        │   ├── Action Buttons
        │   ├── Conversation Log
        │   └── Preview Iframe
        ├── Apps Gallery
        │   ├── Sort Controls
        │   ├── Tag Filters
        │   ├── App Grid
        │   │   └── App Cards (with iframes)
        │   └── Fullscreen Modal
        └── Settings
            ├── Theme Selector
            └── Language Selector
```

---

## Core Components

### State Management

Blackbird uses Dioxus **signals** for reactive state management:

```rust
// Global state signals
let messages = use_signal(Vec::<ChatMessage>::new);
let current_theme = use_signal(|| ThemeMode::Dark);
let saved_apps = use_signal(Vec::<ChatMessage>::new);

// Derived state
let filtered_apps = use_memo(move || {
    apps.iter()
        .filter(|app| matches_tag_filter(app, active_tag))
        .collect()
});
```

### Signal Flow

```
User Action → Signal Update → Component Re-render → DOM Update
     ↓
Side Effects (storage, API calls)
```

### Key Types

```rust
pub struct ChatMessage {
    pub role: Role,           // User or Assistant
    pub content: String,      // Message content (may contain HTML)
    pub timestamp: OffsetDateTime,
    pub tags: Vec<String>,    // Auto-extracted tags
}

pub enum Role {
    User,
    Assistant,
}

pub enum ThemeMode {
    Dark,
    Light,
    Octane,
}
```

---

## Data Flow

### App Generation Flow

```
┌──────────┐    ┌─────────────┐    ┌──────────────┐    ┌────────────┐
│  User    │───▶│  Workbench  │───▶│  AI Client   │───▶│  Provider  │
│  Prompt  │    │  Component  │    │  (Streaming) │    │  (OpenAI)  │
└──────────┘    └──────┬──────┘    └──────┬───────┘    └─────┬──────┘
                       │                  │                  │
                       │                  │◀─────────────────┘
                       │                  │    Stream chunks
                       │                  │
                       ▼                  ▼
               ┌──────────────┐    ┌──────────────┐
               │  Preview     │◀───│  Stream      │
               │  Iframe      │    │  Buffer      │
               └──────────────┘    └──────┬───────┘
                                          │
                                          ▼ (on complete)
                                   ┌──────────────┐
                                   │  Extract     │
                                   │  Tags/Title  │
                                   └──────┬───────┘
                                          │
                                          ▼
                                   ┌──────────────┐
                                   │  Persist to  │
                                   │  Storage     │
                                   └──────────────┘
```

### Streaming Architecture

The streaming system uses a **polling-based approach** for UI compatibility:

```rust
// 1. Initiate stream and get handle
let handle = ai.stream_prompt(prompt).await;

// 2. Store handle in global StreamStore
STREAM_STORE.lock().insert(id, handle);

// 3. Frontend polls every 80ms
use_future(move || async move {
    loop {
        if let Some(chunk) = poll_stream(id) {
            buffer.write().push_str(&chunk);
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
});
```

---

## AI Integration

### Provider Detection

Blackbird automatically selects an AI provider based on available environment variables:

```rust
pub fn detect_provider() -> Provider {
    if env::var("BLACKBIRD_API_KEY").is_ok() {
        Provider::Blackbird
    } else if env::var("OPENAI_API_KEY").is_ok() {
        Provider::OpenAI
    } else if env::var("ANTHROPIC_API_KEY").is_ok() {
        Provider::Anthropic
    } else if env::var("OLLAMA_HOST").is_ok() {
        Provider::Ollama
    } else {
        Provider::default()
    }
}
```

### System Prompt

The AI receives a comprehensive system prompt that includes:

1. **Role definition** — Creative app builder persona
2. **Output format** — Raw HTML/JS/CSS requirements
3. **SDK documentation** — Complete Blackbird storage API reference
4. **Theme integration** — CSS variable usage for consistent styling
5. **Best practices** — App structure guidelines

### Rig Integration

Blackbird uses [Rig](https://github.com/0xPlaygrounds/rig) for unified LLM access:

```rust
use rig::providers::openai;

let client = openai::Client::new(&api_key);
let model = client.agent("gpt-4o")
    .preamble(SYSTEM_PROMPT)
    .build();

let response = model.prompt(user_input).await?;
```

---

## Bridge System

The Bridge system injects the **Blackbird SDK** into every generated app, providing:

### SDK Features

```javascript
// Injected into every app's iframe
window.blackbird = {
    storage: {
        async get(key) { /* ... */ },
        async set(key, value) { /* ... */ },
        async delete(key) { /* ... */ },
        async keys() { /* ... */ },
        async clear() { /* ... */ }
    }
};

// Ready event for app initialization
document.addEventListener('blackbird-ready', () => {
    // SDK is fully loaded
});
```

### Storage Isolation

Each app receives an isolated storage namespace:

```
Storage Root
├── app_abc123/
│   ├── highScore: 9001
│   └── settings: {...}
├── app_def456/
│   └── todos: [...]
└── app_ghi789/
    └── data: {...}
```

### Platform Abstraction

```rust
#[cfg(target_arch = "wasm32")]
fn store_data(key: &str, value: &str) {
    // Use localStorage for web
    web_sys::window()
        .local_storage()
        .set_item(key, value);
}

#[cfg(not(target_arch = "wasm32"))]
fn store_data(key: &str, value: &str) {
    // Use filesystem for native
    let path = get_storage_dir().join(key);
    std::fs::write(path, value);
}
```

---

## Storage Architecture

### Directory Structure

```
Platform-specific app data directory/
└── blackbird/
    ├── apps/
    │   └── saved_apps.json     # App metadata and content
    ├── docs/
    │   └── saved_docs.json     # Document metadata
    └── app_storage/
        └── {app_id}/
            └── {key}.json      # Per-app persistent data
```

### Platform Paths

| Platform | Base Path |
|----------|-----------|
| macOS | `~/Library/Application Support/blackbird/` |
| Linux | `~/.local/share/blackbird/` |
| Windows | `%APPDATA%\blackbird\` |
| iOS | App container sandbox |
| Web | `localStorage` |

### Persistence Format

```json
{
  "apps": [
    {
      "role": "Assistant",
      "content": "<!DOCTYPE html>...",
      "timestamp": "2024-01-15T10:30:00Z",
      "tags": ["calculator", "utility"]
    }
  ]
}
```

---

## Theming System

### Theme Definition

```rust
pub fn get_theme_css(mode: ThemeMode) -> String {
    match mode {
        ThemeMode::Dark => css_vars(
            "--bg": "#0d0d0d",
            "--surface": "#1a1a1a",
            "--text": "#ffffff",
            "--accent": "#3b82f6",
        ),
        ThemeMode::Light => css_vars(
            "--bg": "#ffffff",
            "--surface": "#f5f5f5",
            "--text": "#1a1a1a",
            "--accent": "#2563eb",
        ),
        ThemeMode::Octane => css_vars(
            "--bg": "#0a0a0a",
            "--surface": "#1f1f1f",
            "--text": "#ffffff",
            "--accent": "#f97316",
        ),
    }
}
```

### Theme Injection

Themes are injected into generated apps via CSS variables:

```html
<style>
  :root {
    --bg: #0d0d0d;
    --surface: #1a1a1a;
    --text: #ffffff;
    --accent: #3b82f6;
  }
</style>
<!-- App content follows -->
```

---

## Platform Support

### Build Targets

| Platform | Feature Flag | Compilation Target |
|----------|-------------|-------------------|
| iOS | `mobile` | `aarch64-apple-ios` |
| Desktop | `desktop` | Native (current arch) |
| Web | `web` | `wasm32-unknown-unknown` |

### iOS-Specific Considerations

- Uses Xcode project at `ios/Blackbird.xcodeproj`
- Requires custom build phase for Rust compilation
- Asset bundling via `dx bundle`
- Sandboxed file access within app container

### Web-Specific Considerations

- WASM compilation with wasm-bindgen
- localStorage for persistence
- Limited clipboard access
- Same-origin restrictions for iframes

---

## Performance Considerations

### Streaming Optimization

- 80ms polling interval balances responsiveness with CPU usage
- Chunked updates prevent UI blocking
- Buffer batching reduces DOM operations

### Memory Management

- Apps stored as JSON, loaded on-demand
- Iframe cleanup on view change
- Signal-based reactivity minimizes re-renders

### Startup Time

- 3-second splash screen covers initialization
- Lazy loading of saved apps
- Environment detection cached at startup

---

## Security

### App Sandbox

- Generated apps run in sandboxed iframes
- Each app has isolated storage namespace
- No direct filesystem access from apps
- Cross-origin restrictions enforced

### API Key Management

- Keys loaded from environment variables
- Never exposed to generated apps
- Bundled config for mobile builds (gitignored)

---

## Future Architecture Considerations

- **Multi-conversation support** — Separate message histories
- **Plugin system** — Extensible SDK capabilities
- **App export** — Standalone HTML generation
- **Collaboration** — Real-time shared editing
- **Custom models** — Per-conversation model selection

---

*This document reflects Blackbird v0.1.0-alpha architecture.*
