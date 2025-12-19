//! Integration tests for the Blackbird Bridge SDK
//!
//! Tests storage functionality and SDK script generation

use blackbird::bridge::{
    self, storage_clear, storage_delete, storage_get, storage_keys, storage_set,
};

mod storage_tests {
    use super::*;

    #[test]
    fn test_storage_set_and_get() {
        let app_id = "test-app-1";
        let key = "test_key";
        let value = r#"{"name": "test", "count": 42}"#;

        // Set value
        storage_set(app_id, key, value).expect("Failed to set storage");

        // Get value
        let retrieved = storage_get(app_id, key);
        assert_eq!(retrieved, Some(value.to_string()));

        // Cleanup
        storage_delete(app_id, key).expect("Failed to delete");
    }

    #[test]
    fn test_storage_get_nonexistent() {
        let app_id = "test-app-nonexistent";
        let result = storage_get(app_id, "nonexistent_key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_storage_delete() {
        let app_id = "test-app-delete";
        let key = "to_delete";

        storage_set(app_id, key, "value").expect("Failed to set");
        assert!(storage_get(app_id, key).is_some());

        storage_delete(app_id, key).expect("Failed to delete");
        assert!(storage_get(app_id, key).is_none());
    }

    #[test]
    fn test_storage_keys() {
        let app_id = "test-app-keys";

        // Set multiple keys
        storage_set(app_id, "key1", "value1").expect("Failed to set key1");
        storage_set(app_id, "key2", "value2").expect("Failed to set key2");
        storage_set(app_id, "key3", "value3").expect("Failed to set key3");

        let keys = storage_keys(app_id);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        // Cleanup
        storage_clear(app_id).expect("Failed to clear");
    }

    #[test]
    fn test_storage_clear() {
        let app_id = "test-app-clear";

        storage_set(app_id, "key1", "value1").expect("Failed to set");
        storage_set(app_id, "key2", "value2").expect("Failed to set");

        storage_clear(app_id).expect("Failed to clear");

        assert!(storage_get(app_id, "key1").is_none());
        assert!(storage_get(app_id, "key2").is_none());
        assert!(storage_keys(app_id).is_empty());
    }

    #[test]
    fn test_storage_isolation() {
        let app1 = "test-app-isolation-1";
        let app2 = "test-app-isolation-2";

        storage_set(app1, "shared_key", "app1_value").expect("Failed to set app1");
        storage_set(app2, "shared_key", "app2_value").expect("Failed to set app2");

        assert_eq!(
            storage_get(app1, "shared_key"),
            Some("app1_value".to_string())
        );
        assert_eq!(
            storage_get(app2, "shared_key"),
            Some("app2_value".to_string())
        );

        // Cleanup
        storage_clear(app1).expect("Failed to clear app1");
        storage_clear(app2).expect("Failed to clear app2");
    }

    #[test]
    fn test_storage_special_characters_in_key() {
        let app_id = "test-app-special";
        let key = "user:preferences:theme"; // Contains colons
        let value = "dark";

        storage_set(app_id, key, value).expect("Failed to set");

        // Key gets sanitized, so we need to check with sanitized version
        let keys = storage_keys(app_id);
        assert!(!keys.is_empty());

        storage_clear(app_id).expect("Failed to clear");
    }
}

mod sdk_tests {
    use super::*;

    #[test]
    fn test_sdk_script_contains_app_id() {
        let app_id = "my-test-app";
        let script = bridge::get_sdk_script(app_id);

        assert!(script.contains("my-test-app"));
        assert!(script.contains("const APP_ID"));
        assert!(script.contains("window.blackbird"));
    }

    #[test]
    fn test_sdk_script_has_storage_api() {
        let script = bridge::get_sdk_script("test");

        assert!(script.contains("storage:"));
        assert!(script.contains("get(key)"));
        assert!(script.contains("set(key, value)"));
        assert!(script.contains("delete(key)"));
        assert!(script.contains("keys()"));
        assert!(script.contains("clear()"));
    }

    #[test]
    fn test_sdk_script_has_ai_api() {
        let script = bridge::get_sdk_script("test");

        assert!(script.contains("ai:"));
        assert!(script.contains("chat(prompt)"));
        assert!(script.contains("chatWithHistory(prompt, history)"));
    }

    #[test]
    fn test_sdk_script_has_ready_event() {
        let script = bridge::get_sdk_script("test");

        assert!(script.contains("blackbird:ready"));
        assert!(script.contains("dispatchEvent"));
    }

    #[test]
    fn test_sdk_script_is_wrapped_in_script_tag() {
        let script = bridge::get_sdk_script("test");

        assert!(script.starts_with("<script>"));
        assert!(script.ends_with("</script>"));
    }

    #[test]
    fn test_sdk_storage_prefix() {
        let script = bridge::get_sdk_script("my-app");

        // Should contain storage prefix construction
        assert!(script.contains("STORAGE_PREFIX"));
        assert!(script.contains("blackbird_app_"));
    }
}
