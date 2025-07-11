//! Data models and structures for the Cosmic Noise application.
//!
//! This module contains all the core data structures, domain models, and types
//! used throughout the application. It serves as a single source of truth for
//! data definitions and provides a clean separation between data and business logic.

use kira::sound::PlaybackState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::errors::AppError;

/// Core domain model representing an audio track
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseTrack {
    /// Display name of the track (usually filename without extension)
    pub name: String,
    /// Full file path to the audio file
    pub path: PathBuf,
    /// Current volume level in decibels (-60.0 to 0.0)
    pub volume_level: f32,
    /// Current playback state
    pub state: PlaybackState,
    /// Track metadata (optional)
    pub metadata: Option<TrackMetadata>,
}

impl NoiseTrack {
    /// Create a new noise track with default settings
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            volume_level: DEFAULT_VOLUME_DB,
            state: PlaybackState::Stopped,
            metadata: None,
        }
    }

    /// Create a new track with custom volume
    pub fn with_volume(name: String, path: PathBuf, volume_db: f32) -> Self {
        Self {
            name,
            path,
            volume_level: volume_db.clamp(MIN_VOLUME_DB, MAX_VOLUME_DB),
            state: PlaybackState::Stopped,
            metadata: None,
        }
    }

    /// Check if the track is currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }

    /// Check if the track is currently paused
    pub fn is_paused(&self) -> bool {
        matches!(self.state, PlaybackState::Paused)
    }

    /// Check if the track is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self.state, PlaybackState::Stopped)
    }

    /// Get volume as percentage (0-100)
    pub fn volume_percentage(&self) -> f32 {
        crate::audio::db_to_percentage(self.volume_level)
    }

    /// Set volume from percentage (0-100)
    pub fn set_volume_percentage(&mut self, percentage: f32) {
        self.volume_level = crate::audio::percentage_to_db(percentage);
    }

    /// Get the file extension if available
    pub fn file_extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Check if the file exists on disk
    pub fn file_exists(&self) -> bool {
        self.path.exists()
    }
}

/// Optional metadata for audio tracks
#[derive(Debug, Clone, PartialEq)]
pub struct TrackMetadata {
    /// Track duration in seconds (if known)
    pub duration: Option<f64>,
    /// Audio format (mp3, ogg, flac, wav)
    pub format: Option<AudioFormat>,
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Number of audio channels
    pub channels: Option<u16>,
    /// Bitrate in kbps (for compressed formats)
    pub bitrate: Option<u32>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Last modified timestamp
    pub last_modified: Option<std::time::SystemTime>,
}

impl Default for TrackMetadata {
    fn default() -> Self {
        Self {
            duration: None,
            format: None,
            sample_rate: None,
            channels: None,
            bitrate: None,
            file_size: None,
            last_modified: None,
        }
    }
}

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    Mp3,
    Ogg,
    Flac,
    Wav,
    /// Other supported format
    Other(u32),
}

impl AudioFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::Wav => "wav",
            AudioFormat::Other(_) => "unknown",
        }
    }

    /// Create format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "ogg" => Some(AudioFormat::Ogg),
            "flac" => Some(AudioFormat::Flac),
            "wav" => Some(AudioFormat::Wav),
            _ => None,
        }
    }

    /// Check if format is lossless
    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::Flac | AudioFormat::Wav)
    }
}

/// Audio system configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Default volume level for new tracks (-60.0 to 0.0 dB)
    pub default_volume: f32,
    /// Default fade duration for volume changes
    pub fade_duration: Duration,
    /// Loop settings for ambient sounds
    pub loop_region: Option<std::ops::RangeFrom<f64>>,
    /// Audio buffer size preference
    pub buffer_size: Option<u32>,
    /// Maximum number of simultaneous tracks
    pub max_concurrent_tracks: usize,
    /// Enable audio normalization
    pub normalize_audio: bool,
    /// Master volume level
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
            master_volume: 0.0, // 0 dB master volume
        }
    }
}

/// Audio system statistics and monitoring data
#[derive(Debug, Clone, PartialEq)]
pub struct AudioStats {
    /// Total number of tracks in system
    pub total_tracks: usize,
    /// Number of currently playing tracks
    pub playing_tracks: usize,
    /// Number of currently paused tracks
    pub paused_tracks: usize,
    /// Global playback state
    pub global_state: PlaybackState,
    /// Whether audio system is initialized
    pub is_initialized: bool,
    /// Audio system latency in milliseconds
    pub latency_ms: Option<f32>,
    /// CPU usage percentage for audio processing
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

impl AudioStats {
    /// Check if any tracks are currently active (playing or paused)
    pub fn has_active_tracks(&self) -> bool {
        self.playing_tracks > 0 || self.paused_tracks > 0
    }

    /// Get the total number of loaded tracks
    pub fn loaded_tracks(&self) -> usize {
        self.total_tracks
    }

    /// Check if the audio system is in a healthy state
    pub fn is_healthy(&self) -> bool {
        self.is_initialized && self.cpu_usage.map_or(true, |cpu| cpu < 80.0)
    }
}

/// Application-wide statistics and monitoring data
#[derive(Debug, Clone, PartialEq)]
pub struct AppStats {
    /// Total number of tracks discovered
    pub total_tracks: usize,
    /// Number of successfully loaded tracks
    pub loaded_tracks: usize,
    /// Number of currently playing tracks
    pub playing_tracks: usize,
    /// Number of currently paused tracks
    pub paused_tracks: usize,
    /// Whether there's an active error
    pub has_error: bool,
    /// Whether audio system is initialized
    pub audio_initialized: bool,
    /// Application uptime
    pub uptime: Duration,
    /// Memory usage in MB
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

impl AppStats {
    /// Check if the application is in a good state
    pub fn is_healthy(&self) -> bool {
        !self.has_error && self.audio_initialized && self.loaded_tracks > 0
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        if !self.audio_initialized {
            "Audio system not initialized".to_string()
        } else if self.has_error {
            "Application error occurred".to_string()
        } else if self.loaded_tracks == 0 {
            "No tracks found".to_string()
        } else if self.playing_tracks > 0 {
            format!("Playing {} tracks", self.playing_tracks)
        } else if self.paused_tracks > 0 {
            format!("{} tracks paused", self.paused_tracks)
        } else {
            format!("{} tracks ready", self.loaded_tracks)
        }
    }

    /// Create AppStats from AudioStats
    pub fn from_audio_stats(
        audio_stats: &AudioStats,
        total_tracks: usize,
        has_error: bool,
    ) -> Self {
        Self {
            total_tracks,
            loaded_tracks: total_tracks,
            playing_tracks: audio_stats.playing_tracks,
            paused_tracks: audio_stats.paused_tracks,
            has_error,
            audio_initialized: audio_stats.is_initialized,
            uptime: Duration::from_secs(0), // Will be updated by app
            memory_usage_mb: None,
        }
    }
}

/// Application configuration settings
/// Application configuration that persists between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Current selected theme
    pub theme: AppTheme,
    /// Audio settings
    pub audio: AudioSettings,
    /// UI settings
    pub ui: UiSettings,
    /// File settings
    pub files: FileSettings,
    /// Window settings
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

/// UI-related settings and preferences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    /// UI scale factor
    pub scale_factor: f32,
    /// Show volume as percentage instead of dB
    pub show_volume_percentage: bool,
    /// Enable animations
    pub enable_animations: bool,
    /// Grid layout settings
    pub grid_columns: Option<usize>,
    /// Show track metadata
    pub show_metadata: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            show_volume_percentage: true,
            enable_animations: true,
            grid_columns: None, // Auto-calculate
            show_metadata: false,
        }
    }
}

/// Available application themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    GruvboxLight,
    GruvboxDark,
    #[default]
    Tokyo,
    Catppuccin,
}

impl AppTheme {
    /// Get all available themes
    pub fn all() -> &'static [AppTheme] {
        &[
            AppTheme::GruvboxLight,
            AppTheme::GruvboxDark,
            AppTheme::Tokyo,
            AppTheme::Catppuccin,
        ]
    }

    /// Get theme display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AppTheme::GruvboxLight => "Gruvbox Light",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::Tokyo => "Tokyo Night",
            AppTheme::Catppuccin => "Catppuccin",
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Application view states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum View {
    /// Main player view with track grid
    Player,
    /// Settings view with configuration options
    Settings,
}

impl Default for View {
    fn default() -> Self {
        View::Player
    }
}

/// File system related settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSettings {
    /// Custom directories to scan for audio files
    pub custom_directories: Vec<PathBuf>,
    /// Watch directories for changes
    pub watch_directories: bool,
    /// Supported file extensions
    pub supported_extensions: Vec<String>,
    /// Scan subdirectories recursively
    pub recursive_scan: bool,
    /// Maximum directory scanning depth
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

/// Window-related settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSettings {
    /// Initial window width
    pub width: f32,
    /// Initial window height
    pub height: f32,
    /// Window is resizable
    pub resizable: bool,
    /// Window has decorations
    pub decorations: bool,
    /// Window is transparent
    pub transparent: bool,
    /// Always on top
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

/// Application state information
/// Application lifecycle and state tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// Current application phase
    pub phase: AppPhase,
    /// Last update timestamp
    #[serde(skip, default = "std::time::Instant::now")]
    pub last_update: std::time::Instant,
    /// Application start time
    #[serde(skip, default = "std::time::Instant::now")]
    pub start_time: std::time::Instant,
    /// Current error state
    pub current_error: Option<String>,
    /// Number of tracks loaded
    pub tracks_loaded: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let now = std::time::Instant::now();
        Self {
            phase: AppPhase::Initializing,
            last_update: now,
            start_time: now,
            current_error: None,
            tracks_loaded: 0,
        }
    }
}

impl AppState {
    /// Get application uptime
    pub fn uptime(&self) -> Duration {
        self.last_update.duration_since(self.start_time)
    }

    /// Update the state timestamp
    pub fn touch(&mut self) {
        self.last_update = std::time::Instant::now();
    }

    /// Check if app is in a ready state
    pub fn is_ready(&self) -> bool {
        matches!(self.phase, AppPhase::Ready)
    }
}

/// Application lifecycle phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppPhase {
    /// Application is starting up
    Initializing,
    /// Loading tracks from filesystem
    LoadingTracks,
    /// Initializing audio system
    InitializingAudio,
    /// Application is ready for use
    Ready,
    /// Application is shutting down
    Shutting,
}

// Constants used throughout the application
/// Minimum volume in decibels
pub const MIN_VOLUME_DB: f32 = -60.0;
/// Maximum volume in decibels (safe limit)
pub const MAX_VOLUME_DB: f32 = 0.0;
/// Default volume in decibels
pub const DEFAULT_VOLUME_DB: f32 = -30.0;
/// Supported audio file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "ogg", "flac", "wav"];
/// Default sound directory name
pub const SOUND_DIRECTORY: &str = "cosmic-noise/sounds/";
/// Application name
pub const APP_NAME: &str = "Cosmic Noise";
/// Application version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_track_creation() {
        let track = NoiseTrack::new("test".to_string(), PathBuf::from("/test/path.mp3"));

        assert_eq!(track.name, "test");
        assert_eq!(track.volume_level, DEFAULT_VOLUME_DB);
        assert!(track.is_stopped());
        assert!(!track.is_playing());
    }

    #[test]
    fn test_noise_track_volume() {
        let mut track = NoiseTrack::new("test".to_string(), PathBuf::from("/test/path.mp3"));

        track.set_volume_percentage(50.0);
        assert_eq!(track.volume_percentage(), 50.0);

        // Test clamping
        let track_high = NoiseTrack::with_volume(
            "test".to_string(),
            PathBuf::from("/test/path.mp3"),
            100.0, // Too high
        );
        assert_eq!(track_high.volume_level, MAX_VOLUME_DB);
    }

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("unknown"), None);

        assert!(AudioFormat::Flac.is_lossless());
        assert!(!AudioFormat::Mp3.is_lossless());
    }

    #[test]
    fn test_app_stats() {
        let mut stats = AppStats::default();
        assert!(!stats.is_healthy()); // No audio initialized

        stats.audio_initialized = true;
        stats.loaded_tracks = 5;
        assert!(stats.is_healthy());

        assert_eq!(stats.status_message(), "5 tracks ready");

        stats.playing_tracks = 2;
        assert_eq!(stats.status_message(), "Playing 2 tracks");
    }

    #[test]
    fn test_audio_stats() {
        let stats = AudioStats {
            playing_tracks: 2,
            paused_tracks: 1,
            is_initialized: true,
            ..Default::default()
        };

        assert!(stats.has_active_tracks());
        assert!(stats.is_healthy());
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.audio.default_volume, DEFAULT_VOLUME_DB);
        assert!(config.ui.show_volume_percentage);
        assert_eq!(config.window.width, 800.0);
    }

    #[test]
    fn test_app_state() {
        let mut state = AppState::default();
        assert_eq!(state.phase, AppPhase::Initializing);
        assert!(!state.is_ready());

        state.phase = AppPhase::Ready;
        assert!(state.is_ready());

        let before = state.last_update;
        std::thread::sleep(Duration::from_millis(1));
        state.touch();
        assert!(state.last_update > before);
    }

    #[test]
    fn test_theme_display() {
        assert_eq!(AppTheme::GruvboxLight.display_name(), "Gruvbox Light");
        assert!(AppTheme::all().len() >= 4);
    }
}
