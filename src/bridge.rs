//! Blackbird Bridge - Enables iframe apps to use persistent storage
//!
//! This module provides:
//! - App-specific isolated storage via localStorage

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::PathBuf};

// ============================================
// Storage Backend (for native platforms)
// ============================================

/// In-memory storage for WASM, file-based for native
#[allow(dead_code)]
static APP_STORAGE: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get the storage directory for a specific app
#[cfg(not(target_arch = "wasm32"))]
fn get_app_storage_dir(app_id: &str) -> PathBuf {
    let safe_id = sanitize_app_id(app_id);

    if let Some(data_dir) = dirs::data_local_dir() {
        return data_dir.join("blackbird").join("app_data").join(safe_id);
    }

    PathBuf::from("cache").join("app_data").join(safe_id)
}

/// Sanitize app ID for filesystem use
fn sanitize_app_id(app_id: &str) -> String {
    app_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Get a value from app-specific storage
#[cfg(not(target_arch = "wasm32"))]
pub fn storage_get(app_id: &str, key: &str) -> Option<String> {
    let storage_dir = get_app_storage_dir(app_id);
    let file_path = storage_dir.join(format!("{}.json", sanitize_key(key)));
    fs::read_to_string(file_path).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn storage_get(app_id: &str, key: &str) -> Option<String> {
    let storage = APP_STORAGE.lock().ok()?;
    storage.get(app_id)?.get(key).cloned()
}

/// Set a value in app-specific storage
#[cfg(not(target_arch = "wasm32"))]
pub fn storage_set(app_id: &str, key: &str, value: &str) -> Result<(), String> {
    let storage_dir = get_app_storage_dir(app_id);
    fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    let file_path = storage_dir.join(format!("{}.json", sanitize_key(key)));
    fs::write(file_path, value).map_err(|e| format!("Failed to write to storage: {}", e))
}

#[cfg(target_arch = "wasm32")]
pub fn storage_set(app_id: &str, key: &str, value: &str) -> Result<(), String> {
    let mut storage = APP_STORAGE.lock().map_err(|e| e.to_string())?;
    let app_storage = storage.entry(app_id.to_string()).or_default();
    app_storage.insert(key.to_string(), value.to_string());
    Ok(())
}

/// Delete a value from app-specific storage
#[cfg(not(target_arch = "wasm32"))]
pub fn storage_delete(app_id: &str, key: &str) -> Result<(), String> {
    let storage_dir = get_app_storage_dir(app_id);
    let file_path = storage_dir.join(format!("{}.json", sanitize_key(key)));
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Failed to delete from storage: {}", e))?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn storage_delete(app_id: &str, key: &str) -> Result<(), String> {
    let mut storage = APP_STORAGE.lock().map_err(|e| e.to_string())?;
    if let Some(app_storage) = storage.get_mut(app_id) {
        app_storage.remove(key);
    }
    Ok(())
}

/// List all keys in app-specific storage
#[cfg(not(target_arch = "wasm32"))]
pub fn storage_keys(app_id: &str) -> Vec<String> {
    let storage_dir = get_app_storage_dir(app_id);
    if !storage_dir.exists() {
        return Vec::new();
    }
    fs::read_dir(storage_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn storage_keys(app_id: &str) -> Vec<String> {
    APP_STORAGE
        .lock()
        .ok()
        .and_then(|storage| storage.get(app_id).map(|s| s.keys().cloned().collect()))
        .unwrap_or_default()
}

/// Clear all storage for an app
#[cfg(not(target_arch = "wasm32"))]
pub fn storage_clear(app_id: &str) -> Result<(), String> {
    let storage_dir = get_app_storage_dir(app_id);
    if storage_dir.exists() {
        fs::remove_dir_all(&storage_dir).map_err(|e| format!("Failed to clear storage: {}", e))?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn storage_clear(app_id: &str) -> Result<(), String> {
    let mut storage = APP_STORAGE.lock().map_err(|e| e.to_string())?;
    storage.remove(app_id);
    Ok(())
}

/// Sanitize storage key for filesystem use
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(64)
        .collect()
}

// ============================================
// SDK JavaScript Code
// ============================================

/// The Blackbird SDK that gets injected into every app
/// Currently only provides localStorage-based storage
pub fn get_sdk_script(app_id: &str) -> String {
    format!(
        r#"<script>
(function() {{
    'use strict';

    const APP_ID = '{}';
    const STORAGE_PREFIX = 'blackbird_app_' + APP_ID + '_';

    // Blackbird API
    window.blackbird = {{
        // Storage API - persistent, app-isolated storage via localStorage
        storage: {{
            get(key) {{
                try {{
                    const raw = localStorage.getItem(STORAGE_PREFIX + key);
                    if (raw === null) return null;
                    try {{
                        return JSON.parse(raw);
                    }} catch {{
                        return raw;
                    }}
                }} catch (e) {{
                    console.error('[Blackbird] Storage get error:', e);
                    return null;
                }}
            }},

            set(key, value) {{
                try {{
                    const serialized = typeof value === 'string' ? value : JSON.stringify(value);
                    localStorage.setItem(STORAGE_PREFIX + key, serialized);
                }} catch (e) {{
                    console.error('[Blackbird] Storage set error:', e);
                    throw e;
                }}
            }},

            delete(key) {{
                try {{
                    localStorage.removeItem(STORAGE_PREFIX + key);
                }} catch (e) {{
                    console.error('[Blackbird] Storage delete error:', e);
                }}
            }},

            keys() {{
                try {{
                    const keys = [];
                    for (let i = 0; i < localStorage.length; i++) {{
                        const key = localStorage.key(i);
                        if (key && key.startsWith(STORAGE_PREFIX)) {{
                            keys.push(key.slice(STORAGE_PREFIX.length));
                        }}
                    }}
                    return keys;
                }} catch (e) {{
                    console.error('[Blackbird] Storage keys error:', e);
                    return [];
                }}
            }},

            clear() {{
                try {{
                    const keysToRemove = [];
                    for (let i = 0; i < localStorage.length; i++) {{
                        const key = localStorage.key(i);
                        if (key && key.startsWith(STORAGE_PREFIX)) {{
                            keysToRemove.push(key);
                        }}
                    }}
                    keysToRemove.forEach(key => localStorage.removeItem(key));
                }} catch (e) {{
                    console.error('[Blackbird] Storage clear error:', e);
                }}
            }}
        }},

        // AI API - not yet available
        // Will be implemented in a future update
        ai: {{
            async chat(prompt) {{
                console.warn('[Blackbird] AI API not yet available');
                return 'AI API coming soon!';
            }},

            async chatWithHistory(prompt, history) {{
                console.warn('[Blackbird] AI API not yet available');
                return 'AI API coming soon!';
            }}
        }},

        // App metadata
        app: {{
            id: APP_ID
        }}
    }};

    // Signal SDK is ready
    window.dispatchEvent(new Event('blackbird:ready'));
    console.log('[Blackbird] SDK loaded for app:', APP_ID);
}})();
</script>"#,
        app_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_app_id() {
        assert_eq!(sanitize_app_id("my-app"), "my-app");
        assert_eq!(sanitize_app_id("my app!@#"), "my_app___");
        assert_eq!(sanitize_app_id("/path/to/file.html"), "_path_to_file_html");
    }

    #[test]
    fn test_sanitize_key() {
        assert_eq!(sanitize_key("todos"), "todos");
        assert_eq!(sanitize_key("user:preferences"), "user_preferences");
    }
}
