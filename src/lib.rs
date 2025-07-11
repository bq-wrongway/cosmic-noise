//! Cosmic Noise - Ambient noise player library
//!
//! This library provides the core functionality for the Cosmic Noise application,
//! including audio playback, configuration management, and UI components.

pub mod app;
pub mod audio;
pub mod config;
pub mod errors;
pub mod i18n;
pub mod messages;
pub mod models;

pub mod ui;
pub mod utils;

// Constants used throughout the application
pub const SPACING: f32 = 5.0;

// Re-export commonly used types for convenience
pub use app::{CosmicNoise, Message};
pub use config::ConfigManager;
pub use errors::{AppError, AudioError, ConfigError, FileSystemError, UIError};
pub use models::{AppConfig, AppTheme, NoiseTrack, View};
