use crate::types::ThemeMode;

pub struct ThemeDefinition {
    pub css: &'static str,
    pub wordmark_class: &'static str,
}

pub fn theme_definition(mode: ThemeMode) -> ThemeDefinition {
    match mode {
        ThemeMode::Dark => ThemeDefinition {
            css: DARK_THEME,
            wordmark_class: "header-wordmark",
        },
        ThemeMode::Light => ThemeDefinition {
            css: LIGHT_THEME,
            wordmark_class: "header-wordmark",
        },
        ThemeMode::Octane => ThemeDefinition {
            css: OCTANE_THEME,
            wordmark_class: "header-wordmark header-wordmark-octane",
        },
    }
}

const DARK_THEME: &str = r#"
:root {
    --color-bg-primary: #000000;
    --color-bg-secondary: #050505;
    --color-bg-overlay: rgba(0, 0, 0, 0.9);
    --color-text-primary: #ffffff;
    --color-text-secondary: #ffffff;
    --color-text-muted: #cfcfcf;
    --color-border: #ffffff;
    --color-surface-muted: #111111;
    --color-input-border: #2a2a2a;
    --color-input-bg: #000000;
    --color-chat-user-bg: #ffffff;
    --color-chat-user-text: #000000;
    --color-chat-assistant-bg: #000000;
    --color-chat-assistant-text: #ffffff;
    --color-doc-card-border: #2a2a2a;
    --color-doc-card-bg: #050505;
    --color-doc-card-hover: #ffffff;
    --color-doc-viewer-bg: rgba(255, 255, 255, 0.02);
    --color-timestamp: #9b9b9b;
    --color-shimmer-base: rgba(255, 53, 9, 0.25);
    --color-shimmer-highlight: #ff3509;
    --color-header-fade: rgba(0, 0, 0, 0.85);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: var(--color-bg-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: transparent; }
"#;

const LIGHT_THEME: &str = r#"
:root {
    --color-bg-primary: #ffffff;
    --color-bg-secondary: #f5f5f5;
    --color-bg-overlay: rgba(255, 255, 255, 0.92);
    --color-text-primary: #000000;
    --color-text-secondary: #000000;
    --color-text-muted: #4a4a4a;
    --color-border: #000000;
    --color-surface-muted: #e6e6e6;
    --color-input-border: #c2c2c2;
    --color-input-bg: #ffffff;
    --color-chat-user-bg: #111111;
    --color-chat-user-text: #ffffff;
    --color-chat-assistant-bg: #ffffff;
    --color-chat-assistant-text: #000000;
    --color-doc-card-border: #d0d0d0;
    --color-doc-card-bg: #f5f5f5;
    --color-doc-card-hover: #000000;
    --color-doc-viewer-bg: rgba(0, 0, 0, 0.04);
    --color-timestamp: #606060;
    --color-shimmer-base: rgba(255, 53, 9, 0.25);
    --color-shimmer-highlight: #ff3509;
    --color-header-fade: rgba(255, 255, 255, 0.9);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: var(--color-bg-primary); }
.btn { color: var(--color-text-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.composer { background: var(--color-bg-overlay); border-top-color: var(--color-border); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: var(--color-doc-viewer-bg); }
"#;

const OCTANE_THEME: &str = r#"
:root {
    --color-bg-primary: #ff3509;
    --color-bg-secondary: #ffe0d1;
    --color-bg-overlay: rgba(0, 0, 0, 0.55);
    --color-text-primary: #000000;
    --color-text-secondary: #111111;
    --color-text-muted: #2c2c2c;
    --color-border: #000000;
    --color-surface-muted: rgba(0, 0, 0, 0.12);
    --color-input-border: rgba(0, 0, 0, 0.6);
    --color-input-bg: #ffe8de;
    --color-chat-user-bg: #000000;
    --color-chat-user-text: #ffffff;
    --color-chat-assistant-bg: rgba(255, 255, 255, 0.92);
    --color-chat-assistant-text: #121212;
    --color-doc-card-border: rgba(0, 0, 0, 0.5);
    --color-doc-card-bg: #ffb79f;
    --color-doc-card-hover: #000000;
    --color-doc-viewer-bg: rgba(255, 240, 234, 0.96);
    --color-timestamp: rgba(0, 0, 0, 0.75);
    --color-shimmer-base: rgba(255, 255, 255, 0.3);
    --color-shimmer-highlight: #ffffff;
    --color-accent-primary: #ffffff;
    --color-header-fade: rgba(255, 53, 9, 0.92);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: transparent; }
.btn { color: var(--color-text-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.btn-primary { border-color: var(--color-accent-primary); color: #000; }
.btn-primary:hover { background: var(--color-accent-primary); color: #000; }
.composer { background: var(--color-bg-overlay); border-top-color: var(--color-border); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: var(--color-doc-viewer-bg); }
"#;
