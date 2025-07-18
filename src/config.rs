use crate::errors::{AppError, ConfigError};
use crate::models::{AppConfig, AppTheme};
use log::{error, info, warn};

// Application information for confy
const APP_NAME: &str = "cosmic-noise";
const CONFIG_NAME: &str = "config";

// Configuration manager for the application
pub struct ConfigManager;

impl ConfigManager {
    // Load configuration from disk, or create default if it doesn't exist
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

    // Save configuration to disk
    pub fn save(config: &AppConfig) -> Result<(), AppError> {
        confy::store(APP_NAME, CONFIG_NAME, config).map_err(|e| {
            error!("Failed to save configuration: {e}");
            AppError::Config(ConfigError::SaveFailed)
        })?;

        info!("Configuration saved successfully");
        Ok(())
    }

    // Load only the theme from configuration
    pub fn load_theme() -> AppTheme {
        match Self::load() {
            Ok(config) => config.theme,
            Err(e) => {
                warn!("Failed to load theme from configuration: {e}");
                AppTheme::default()
            }
        }
    }

    // Save only the theme to configuration
    pub fn save_theme(theme: AppTheme) -> Result<(), AppError> {
        let mut config = Self::load().unwrap_or_default();
        config.theme = theme;
        Self::save(&config)
    }

    // Load only the master volume from configuration
    pub fn load_master_volume() -> f32 {
        match Self::load() {
            Ok(config) => config.audio.master_volume,
            Err(e) => {
                warn!("Failed to load master volume from configuration: {e}");
                crate::models::DEFAULT_VOLUME_DB
            }
        }
    }

    // Save only the master volume to configuration
    pub fn save_master_volume(volume: f32) -> Result<(), AppError> {
        let mut config = Self::load().unwrap_or_default();
        config.audio.master_volume = volume;
        Self::save(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading_defaults() {
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
}
