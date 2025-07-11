//! Configuration management for the Cosmic Noise application.
//!
//! This module handles loading, saving, and managing application configuration
//! using the confy crate for cross-platform configuration persistence.

use crate::errors::{AppError, ConfigError};
use crate::models::{AppConfig, AppTheme};
use log::{error, info, warn};

/// Application information for confy
const APP_NAME: &str = "cosmic-noise";
const CONFIG_NAME: &str = "config";

/// Configuration manager for the application
pub struct ConfigManager;

impl ConfigManager {
    /// Load configuration from disk, or create default if it doesn't exist
    pub fn load() -> Result<AppConfig, AppError> {
        match confy::load(APP_NAME, CONFIG_NAME) {
            Ok(config) => {
                info!("Configuration loaded successfully from disk");
                Ok(config)
            }
            Err(e) => {
                warn!("Failed to load configuration: {e}, using defaults");
                // Return default configuration if loading fails
                let default_config = AppConfig::default();
                // Try to save the default configuration
                if let Err(save_err) = Self::save(&default_config) {
                    error!("Failed to save default configuration: {save_err}");
                }
                Ok(default_config)
            }
        }
    }

    /// Save configuration to disk
    pub fn save(config: &AppConfig) -> Result<(), AppError> {
        confy::store(APP_NAME, CONFIG_NAME, config).map_err(|e| {
            error!("Failed to save configuration: {e}");
            AppError::Config(ConfigError::SaveFailed)
        })?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Load only the theme from configuration
    pub fn load_theme() -> AppTheme {
        match Self::load() {
            Ok(config) => config.theme,
            Err(e) => {
                warn!("Failed to load theme from configuration: {e}");
                AppTheme::default()
            }
        }
    }

    /// Save only the theme to configuration
    pub fn save_theme(theme: AppTheme) -> Result<(), AppError> {
        let mut config = Self::load().unwrap_or_default();
        config.theme = theme;
        Self::save(&config)
    }

    // Get the configuration file path
    // pub fn get_config_path() -> Option<std::path::PathBuf> {
    //     confy::get_configuration_file_path(APP_NAME, CONFIG_NAME).ok()
    // }

    // Reset configuration to defaults
    // pub fn reset_to_defaults() -> Result<(), AppError> {
    //     let default_config = AppConfig::default();
    //     Self::save(&default_config)?;
    //     info!("Configuration reset to defaults");
    //     Ok(())
    // }

    // Check if configuration file exists
    // pub fn exists() -> bool {
    //     if let Some(path) = Self::get_config_path() {
    //         path.exists()
    //     } else {
    //         false
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading_defaults() {
        // Test that loading config returns defaults when file doesn't exist
        // This is a basic test since actual file I/O depends on the environment
        let config = AppConfig::default();
        assert_eq!(config.theme, AppTheme::GruvboxLight);
        assert!(config.ui.show_volume_percentage);
        assert!(config.ui.enable_animations);
        assert_eq!(config.ui.grid_columns, None);
    }

    #[test]
    fn test_theme_default() {
        let theme = AppTheme::default();
        assert_eq!(theme, AppTheme::Tokyo);
    }

    // #[test]
    // fn test_config_manager_methods() {
    //     // Test that ConfigManager methods can be called without panicking
    //     let _exists = ConfigManager::exists();
    //     let _path = ConfigManager::get_config_path();
    //     let _theme = ConfigManager::load_theme();

    //     // These tests pass if no panic occurs
    // }
}
