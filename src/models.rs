use kira::sound::PlaybackState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

// Core domain model representing an audio track
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseTrack {
    // Display name of the track (usually filename without extension)
    pub name: String,
    // Full file path to the audio file
    pub path: PathBuf,
    // Current volume level in decibels (-60.0 to 0.0)
    pub volume_level: f32,
    // Current playback state
    pub state: PlaybackState,
    // Track metadata (optional)
    pub metadata: Option<TrackMetadata>,
}

impl NoiseTrack {
    // Create a new noise track with default settings
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            volume_level: DEFAULT_VOLUME_DB,
            state: PlaybackState::Stopped,
            metadata: None,
        }
    }
}

// Optional metadata for audio tracks
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackMetadata {
    // Track duration in seconds (if known)
    pub duration: Option<f64>,
    // Audio format (mp3, ogg, flac, wav)
    pub format: Option<String>,
    // Sample rate in Hz
    pub sample_rate: Option<u32>,
    // Number of audio channels
    pub channels: Option<u16>,
    // Bitrate in kbps (for compressed formats)
    pub bitrate: Option<u32>,
    // File size in bytes
    pub file_size: Option<u64>,
    // Last modified timestamp
    pub last_modified: Option<std::time::SystemTime>,
}

// Audio system configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    // Default volume level for new tracks (-60.0 to 0.0 dB)
    pub default_volume: f32,
    // Default fade duration for volume changes
    pub fade_duration: Duration,
    // Loop settings for ambient sounds
    pub loop_region: Option<std::ops::RangeFrom<f64>>,
    // Audio buffer size preference
    pub buffer_size: Option<u32>,
    // Maximum number of simultaneous tracks
    pub max_concurrent_tracks: usize,
    // Enable audio normalization
    pub normalize_audio: bool,
    // Master volume level
    pub master_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            default_volume: DEFAULT_VOLUME_DB,
            fade_duration: Duration::from_secs(1),
            loop_region: Some(0.0..),
            buffer_size: None,
            max_concurrent_tracks: 16,
            normalize_audio: false,
            master_volume: DEFAULT_VOLUME_DB, // Start at 50% like other sliders
        }
    }
}

// Audio system statistics and monitoring data
#[derive(Debug, Clone, PartialEq)]
pub struct AudioStats {
    // Total number of tracks in system
    pub total_tracks: usize,
    // Number of currently playing tracks
    pub playing_tracks: usize,
    // Number of currently paused tracks
    pub paused_tracks: usize,
    // Global playback state
    pub global_state: PlaybackState,
    // Whether audio system is initialized
    pub is_initialized: bool,
    // Audio system latency in milliseconds
    pub latency_ms: Option<f32>,
    // CPU usage percentage for audio processing
    pub cpu_usage: Option<f32>,
}

impl Default for AudioStats {
    fn default() -> Self {
        Self {
            total_tracks: 0,
            playing_tracks: 0,
            paused_tracks: 0,
            global_state: PlaybackState::Stopped,
            is_initialized: false,
            latency_ms: None,
            cpu_usage: None,
        }
    }
}

// Application-wide statistics and monitoring data
#[derive(Debug, Clone, PartialEq)]
pub struct AppStats {
    // Total number of tracks discovered
    pub total_tracks: usize,
    // Number of successfully loaded tracks
    pub loaded_tracks: usize,
    // Number of currently playing tracks
    pub playing_tracks: usize,
    // Number of currently paused tracks
    pub paused_tracks: usize,
    // Whether there's an active error
    pub has_error: bool,
    // Whether audio system is initialized
    pub audio_initialized: bool,
    // Application uptime
    pub uptime: Duration,
    // Memory usage in MB
    pub memory_usage_mb: Option<f32>,
}

impl Default for AppStats {
    fn default() -> Self {
        Self {
            total_tracks: 0,
            loaded_tracks: 0,
            playing_tracks: 0,
            paused_tracks: 0,
            has_error: false,
            audio_initialized: false,
            uptime: Duration::from_secs(0),
            memory_usage_mb: None,
        }
    }
}

// Application configuration settings
// Application configuration that persists between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Current selected theme
    pub theme: AppTheme,
    // Audio settings
    pub audio: AudioSettings,
    // UI settings
    pub ui: UiSettings,
    // File settings
    pub files: FileSettings,
    // Window settings
    pub window: WindowSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: AppTheme::GruvboxLight,
            audio: AudioSettings::default(),
            ui: UiSettings::default(),
            files: FileSettings::default(),
            window: WindowSettings::default(),
        }
    }
}

// UI-related settings and preferences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    // UI scale factor
    pub scale_factor: f32,
    // Show volume as percentage instead of dB
    pub show_volume_percentage: bool,
    // Enable animations
    pub enable_animations: bool,
    // Grid layout settings
    pub grid_columns: Option<usize>,
    // Show track metadata
    pub show_metadata: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            show_volume_percentage: true,
            enable_animations: true,
            grid_columns: None,
            show_metadata: false,
        }
    }
}

// Available application themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    Light,
    GruvboxLight,
    GruvboxDark,
    #[default]
    Tokyo,
    Catppuccin,
    Moonfly,
}

impl AppTheme {
    // Get all available themes
    pub fn all() -> &'static [AppTheme] {
        &[
            AppTheme::Light,
            AppTheme::GruvboxLight,
            AppTheme::GruvboxDark,
            AppTheme::Tokyo,
            AppTheme::Catppuccin,
            AppTheme::Moonfly,
        ]
    }

    // Get theme display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AppTheme::Light => "Light",
            AppTheme::GruvboxLight => "Gruvbox Light",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::Tokyo => "Tokyo Night",
            AppTheme::Catppuccin => "Catppuccin",
            AppTheme::Moonfly => "Moonfly",
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// Application view states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum View {
    // Main player view with track grid
    #[default]
    Player,
    // Settings view with configuration options
    Settings,
}

// File system related settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSettings {
    // Custom directories to scan for audio files
    pub custom_directories: Vec<PathBuf>,
    // Watch directories for changes
    pub watch_directories: bool,
    // Supported file extensions
    pub supported_extensions: Vec<String>,
    // Scan subdirectories recursively
    pub recursive_scan: bool,
    // Maximum directory scanning depth
    pub max_scan_depth: usize,
}

impl Default for FileSettings {
    fn default() -> Self {
        Self {
            custom_directories: vec![],
            watch_directories: false,
            supported_extensions: SUPPORTED_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            recursive_scan: true,
            max_scan_depth: 3,
        }
    }
}

// Window-related settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSettings {
    // Initial window width
    pub width: f32,
    // Initial window height
    pub height: f32,
    // Window is resizable
    pub resizable: bool,
    // Window has decorations
    pub decorations: bool,
    // Window is transparent
    pub transparent: bool,
    // Always on top
    pub always_on_top: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 650.0,
            resizable: true,
            decorations: false,
            transparent: true,
            always_on_top: false,
        }
    }
}

// Constants used throughout the application
// Default volume in decibels
pub const DEFAULT_VOLUME_DB: f32 = -30.0;
#[allow(dead_code)]
pub const MAX_VOLUME_DB: f32 = 0.0;
#[allow(dead_code)]
pub const MIN_VOLUME_DB: f32 = -60.0;
// Supported audio file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "ogg", "flac", "wav"];
// Default sound directory name
pub const SOUND_DIRECTORY: &str = "cosmic-noise/sounds";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_track_creation() {
        let track = NoiseTrack::new("test".to_string(), PathBuf::from("/test/path.mp3"));

        assert_eq!(track.name, "test");
        assert_eq!(track.volume_level, DEFAULT_VOLUME_DB);
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.audio.default_volume, DEFAULT_VOLUME_DB);
        assert!(config.ui.show_volume_percentage);
        assert_eq!(config.window.width, 800.0);
    }

    #[test]
    fn test_theme_display() {
        assert_eq!(AppTheme::GruvboxLight.display_name(), "Gruvbox Light");
        assert!(AppTheme::all().len() >= 4);
    }
}
