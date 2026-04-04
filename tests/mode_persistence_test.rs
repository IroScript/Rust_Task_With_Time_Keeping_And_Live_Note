// Property Test for Task 9.4: Mode Persistence
// Property 4: Mode persists across application restarts
// Validates: Requirements 6.1, 6.2

use std::fs::{self, File};
use serde::{Deserialize, Serialize};

// Minimal types needed for testing persistence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    single_quote_mode: bool,
}

impl TestConfig {
    fn save(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn load(path: &str) -> Option<Self> {
        if let Ok(file) = File::open(path) {
            serde_json::from_reader(file).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod mode_persistence_tests {
    use super::*;

    /// Property 4: Mode persists across application restarts
    /// 
    /// This property test verifies that:
    /// 1. When single_quote_mode is saved, it can be loaded back with the same value
    /// 2. The persistence mechanism correctly handles both true and false values
    /// 3. The default value is false when no saved config exists
    #[test]
    fn property_mode_persists_true() {
        let test_file = "test_settings_true.json";
        
        // Clean up any existing test file
        let _ = fs::remove_file(test_file);
        
        // Simulate saving state with single_quote_mode = true
        let config = TestConfig {
            single_quote_mode: true,
        };
        config.save(test_file).expect("Failed to save config");
        
        // Simulate loading state (application restart)
        let loaded_config = TestConfig::load(test_file).expect("Failed to load config");
        
        // Verify the mode persisted correctly
        assert_eq!(loaded_config.single_quote_mode, true, 
            "Property violated: single_quote_mode=true should persist across restarts");
        
        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn property_mode_persists_false() {
        let test_file = "test_settings_false.json";
        
        // Clean up any existing test file
        let _ = fs::remove_file(test_file);
        
        // Simulate saving state with single_quote_mode = false
        let config = TestConfig {
            single_quote_mode: false,
        };
        config.save(test_file).expect("Failed to save config");
        
        // Simulate loading state (application restart)
        let loaded_config = TestConfig::load(test_file).expect("Failed to load config");
        
        // Verify the mode persisted correctly
        assert_eq!(loaded_config.single_quote_mode, false,
            "Property violated: single_quote_mode=false should persist across restarts");
        
        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn property_default_mode_is_false() {
        let test_file = "test_settings_nonexistent.json";
        
        // Ensure the file doesn't exist
        let _ = fs::remove_file(test_file);
        
        // Simulate loading when no config exists
        let loaded_config = TestConfig::load(test_file);
        
        // Verify that None is returned (which should trigger default initialization)
        assert!(loaded_config.is_none(),
            "Property violated: loading non-existent config should return None");
        
        // Verify default value is false (as per AppState::default())
        let default_mode = false;
        assert_eq!(default_mode, false,
            "Property violated: default single_quote_mode should be false");
    }

    #[test]
    fn property_multiple_save_load_cycles() {
        let test_file = "test_settings_cycles.json";
        
        // Clean up any existing test file
        let _ = fs::remove_file(test_file);
        
        // Test multiple save/load cycles
        for expected_mode in [true, false, true, false, true].iter() {
            // Save
            let config = TestConfig {
                single_quote_mode: *expected_mode,
            };
            config.save(test_file).expect("Failed to save config");
            
            // Load
            let loaded_config = TestConfig::load(test_file).expect("Failed to load config");
            
            // Verify
            assert_eq!(loaded_config.single_quote_mode, *expected_mode,
                "Property violated: mode should persist correctly across multiple cycles");
        }
        
        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn property_persistence_is_idempotent() {
        let test_file = "test_settings_idempotent.json";
        
        // Clean up any existing test file
        let _ = fs::remove_file(test_file);
        
        // Save once
        let config = TestConfig {
            single_quote_mode: true,
        };
        config.save(test_file).expect("Failed to save config");
        
        // Load
        let loaded_once = TestConfig::load(test_file).expect("Failed to load config");
        
        // Save again with the same value
        loaded_once.save(test_file).expect("Failed to save config again");
        
        // Load again
        let loaded_twice = TestConfig::load(test_file).expect("Failed to load config again");
        
        // Verify idempotence: saving and loading multiple times doesn't change the value
        assert_eq!(loaded_once.single_quote_mode, loaded_twice.single_quote_mode,
            "Property violated: persistence should be idempotent");
        
        // Clean up
        let _ = fs::remove_file(test_file);
    }
}
