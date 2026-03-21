//! Persistent configuration file management
//!
//! This module handles saving and loading configuration to a JSON file
//! so that settings persist across restarts.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Persistent user configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub embedding_provider: Option<String>,
    pub embedding_base_url: Option<String>,
    pub embedding_model: Option<String>,
    pub maintenance_interval_hours: Option<i32>,
    pub auto_decay_enabled: Option<bool>,
    pub auto_flush_enabled: Option<bool>,
    pub min_importance_threshold: Option<f64>,
    pub api_key: Option<String>,
}

impl UserConfig {
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Path::new("ChetnaData").join("config.json")
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let config: UserConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Update and save configuration
    pub fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self),
    {
        f(self);
        self.save()
    }
}

/// Sync config.json to .env file for backward compatibility
pub fn sync_to_env(config: &UserConfig) -> Result<()> {
    let env_path = Path::new("ChetnaData/.env");

    // Read existing content
    let content = if env_path.exists() {
        fs::read_to_string(env_path)?
    } else {
        String::new()
    };

    let mut new_content = String::new();
    let mut updated_keys = std::collections::HashSet::new();

    // Process existing lines
    for line in content.lines() {
        let mut modified = false;

        if line.starts_with("EMBEDDING_BASE_URL=") {
            if let Some(url) = &config.embedding_base_url {
                new_content.push_str(&format!("EMBEDDING_BASE_URL={}\n", url));
                modified = true;
                updated_keys.insert("EMBEDDING_BASE_URL");
            }
        } else if line.starts_with("EMBEDDING_MODEL=") {
            if let Some(model) = &config.embedding_model {
                new_content.push_str(&format!("EMBEDDING_MODEL={}\n", model));
                modified = true;
                updated_keys.insert("EMBEDDING_MODEL");
            }
        } else if line.starts_with("MAINTENANCE_INTERVAL=") {
            if let Some(interval) = config.maintenance_interval_hours {
                new_content.push_str(&format!("MAINTENANCE_INTERVAL={}\n", interval));
                modified = true;
                updated_keys.insert("MAINTENANCE_INTERVAL");
            }
        } else if line.starts_with("AUTO_DECAY_ENABLED=") {
            if let Some(enabled) = config.auto_decay_enabled {
                new_content.push_str(&format!("AUTO_DECAY_ENABLED={}\n", enabled));
                modified = true;
                updated_keys.insert("AUTO_DECAY_ENABLED");
            }
        } else if line.starts_with("AUTO_FLUSH_ENABLED=") {
            if let Some(enabled) = config.auto_flush_enabled {
                new_content.push_str(&format!("AUTO_FLUSH_ENABLED={}\n", enabled));
                modified = true;
                updated_keys.insert("AUTO_FLUSH_ENABLED");
            }
        } else if line.starts_with("MIN_IMPORTANCE_THRESHOLD=") {
            if let Some(threshold) = config.min_importance_threshold {
                new_content.push_str(&format!("MIN_IMPORTANCE_THRESHOLD={}\n", threshold));
                modified = true;
                updated_keys.insert("MIN_IMPORTANCE_THRESHOLD");
            }
        } else if line.starts_with("CHETNA_API_KEY=") {
            if let Some(key) = &config.api_key {
                new_content.push_str(&format!("CHETNA_API_KEY={}\n", key));
                modified = true;
                updated_keys.insert("CHETNA_API_KEY");
            }
        }

        if !modified {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Add any missing entries
    if let Some(url) = &config.embedding_base_url {
        if !updated_keys.contains("EMBEDDING_BASE_URL") {
            new_content.push_str(&format!("EMBEDDING_BASE_URL={}\n", url));
        }
    }
    if let Some(model) = &config.embedding_model {
        if !updated_keys.contains("EMBEDDING_MODEL") {
            new_content.push_str(&format!("EMBEDDING_MODEL={}\n", model));
        }
    }
    if let Some(interval) = config.maintenance_interval_hours {
        if !updated_keys.contains("MAINTENANCE_INTERVAL") {
            new_content.push_str(&format!("MAINTENANCE_INTERVAL={}\n", interval));
        }
    }
    if let Some(enabled) = config.auto_decay_enabled {
        if !updated_keys.contains("AUTO_DECAY_ENABLED") {
            new_content.push_str(&format!("AUTO_DECAY_ENABLED={}\n", enabled));
        }
    }
    if let Some(enabled) = config.auto_flush_enabled {
        if !updated_keys.contains("AUTO_FLUSH_ENABLED") {
            new_content.push_str(&format!("AUTO_FLUSH_ENABLED={}\n", enabled));
        }
    }
    if let Some(threshold) = config.min_importance_threshold {
        if !updated_keys.contains("MIN_IMPORTANCE_THRESHOLD") {
            new_content.push_str(&format!("MIN_IMPORTANCE_THRESHOLD={}\n", threshold));
        }
    }
    if let Some(key) = &config.api_key {
        if !updated_keys.contains("CHETNA_API_KEY") {
            new_content.push_str(&format!("CHETNA_API_KEY={}\n", key));
        }
    }

    fs::write(env_path, new_content)?;
    Ok(())
}
