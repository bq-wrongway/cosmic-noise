//! Error handling for the Cosmic Noise application.
//!
//! This module provides comprehensive error types and handling for all application components
//! including file system operations, audio playback, and UI interactions.

use std::fmt;

/// Main error type for the Cosmic Noise application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    /// File system related errors
    FileSystem(FileSystemError),
    /// Audio playback related errors
    Audio(AudioError),
    /// Configuration related errors
    Config(ConfigError),
    /// UI related errors
    UI(UIError),
}

/// File system related errors
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemError {
    /// Could not find sound directory
    DirectoryNotFound,
    /// Could not read directory contents
    DirectoryReadError,
    /// Invalid file format
    InvalidFileFormat,
    /// File access permission denied
    PermissionDenied,
    /// Generic IO error
    IOError(String),
}

/// Audio playback related errors
#[derive(Debug, Clone, PartialEq)]
pub enum AudioError {
    /// Failed to initialize audio manager
    InitializationFailed,
    /// Failed to create audio handle
    HandleCreationFailed,
    /// Audio file has no default track
    NoDefaultTrack,
    /// Unknown or unsupported sample rate
    UnknownSampleRate,
    /// Unknown duration in audio file
    UnknownDuration,
    /// Unsupported channel configuration
    UnsupportedChannelConfiguration,
    /// Symphonia decoder error
    DecoderError(String),
    /// Audio device not available
    DeviceNotAvailable,
    /// Playback error during runtime
    PlaybackError(String),
}

/// Configuration related errors
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Failed to load configuration
    LoadFailed,
    /// Failed to save configuration
    SaveFailed,
    /// Invalid configuration format
    InvalidFormat,
    /// Missing required configuration key
    MissingKey(String),
}

/// UI related errors
#[derive(Debug, Clone, PartialEq)]
pub enum UIError {
    /// Failed to render component
    RenderFailed,
    /// Invalid UI state
    InvalidState,
    /// Theme loading failed
    ThemeLoadFailed,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FileSystem(e) => write!(f, "File system error: {}", e),
            AppError::Audio(e) => write!(f, "Audio error: {}", e),
            AppError::Config(e) => write!(f, "Configuration error: {}", e),
            AppError::UI(e) => write!(f, "UI error: {}", e),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::DirectoryNotFound => {
                write!(
                    f,
                    "Sound directory not found. Please check that sounds exist in '$HOME/.local/share/cosmic-noise/sounds' or '$HOME/.config/cosmic-noise/sounds'"
                )
            }
            FileSystemError::DirectoryReadError => {
                write!(f, "Could not read directory contents")
            }
            FileSystemError::InvalidFileFormat => {
                write!(f, "Invalid or unsupported audio file format")
            }
            FileSystemError::PermissionDenied => {
                write!(f, "Permission denied accessing audio files")
            }
            FileSystemError::IOError(msg) => {
                write!(f, "IO error: {}", msg)
            }
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::InitializationFailed => {
                write!(f, "Failed to initialize audio system")
            }
            AudioError::HandleCreationFailed => {
                write!(f, "Failed to create audio playback handle")
            }
            AudioError::NoDefaultTrack => {
                write!(f, "Audio file contains no default track")
            }
            AudioError::UnknownSampleRate => {
                write!(f, "Audio file has unknown or unsupported sample rate")
            }
            AudioError::UnknownDuration => {
                write!(f, "Could not determine audio file duration")
            }
            AudioError::UnsupportedChannelConfiguration => {
                write!(f, "Audio file has unsupported channel configuration")
            }
            AudioError::DecoderError(msg) => {
                write!(f, "Audio decoder error: {}", msg)
            }
            AudioError::DeviceNotAvailable => {
                write!(f, "Audio device not available")
            }
            AudioError::PlaybackError(msg) => {
                write!(f, "Playback error: {}", msg)
            }
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::LoadFailed => {
                write!(f, "Failed to load configuration")
            }
            ConfigError::SaveFailed => {
                write!(f, "Failed to save configuration")
            }
            ConfigError::InvalidFormat => {
                write!(f, "Invalid configuration format")
            }
            ConfigError::MissingKey(key) => {
                write!(f, "Missing required configuration key: {}", key)
            }
        }
    }
}

impl fmt::Display for UIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UIError::RenderFailed => {
                write!(f, "Failed to render UI component")
            }
            UIError::InvalidState => {
                write!(f, "Invalid UI state")
            }
            UIError::ThemeLoadFailed => {
                write!(f, "Failed to load theme")
            }
        }
    }
}

impl std::error::Error for AppError {}
impl std::error::Error for FileSystemError {}
impl std::error::Error for AudioError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for UIError {}

/// Convenience type alias for Results using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Helper functions for error conversion
impl From<FileSystemError> for AppError {
    fn from(error: FileSystemError) -> Self {
        AppError::FileSystem(error)
    }
}

impl From<AudioError> for AppError {
    fn from(error: AudioError) -> Self {
        AppError::Audio(error)
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        AppError::Config(error)
    }
}

impl From<UIError> for AppError {
    fn from(error: UIError) -> Self {
        AppError::UI(error)
    }
}

/// Convert from Kira's FromFileError to our AudioError
impl From<kira::sound::FromFileError> for AudioError {
    fn from(error: kira::sound::FromFileError) -> Self {
        match error {
            kira::sound::FromFileError::NoDefaultTrack => AudioError::NoDefaultTrack,
            kira::sound::FromFileError::UnknownSampleRate => AudioError::UnknownSampleRate,
            kira::sound::FromFileError::UnknownDuration => AudioError::UnknownDuration,
            kira::sound::FromFileError::UnsupportedChannelConfiguration => {
                AudioError::UnsupportedChannelConfiguration
            }
            kira::sound::FromFileError::IoError(e) => AudioError::PlaybackError(e.to_string()),
            kira::sound::FromFileError::SymphoniaError(e) => {
                AudioError::DecoderError(e.to_string())
            }
        }
    }
}

/// Convert from std::io::Error to FileSystemError
impl From<std::io::Error> for FileSystemError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => FileSystemError::DirectoryNotFound,
            std::io::ErrorKind::PermissionDenied => FileSystemError::PermissionDenied,
            _ => FileSystemError::IOError(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let fs_error = AppError::FileSystem(FileSystemError::DirectoryNotFound);
        assert!(fs_error.to_string().contains("Sound directory not found"));

        let audio_error = AppError::Audio(AudioError::InitializationFailed);
        assert!(
            audio_error
                .to_string()
                .contains("Failed to initialize audio system")
        );
    }

    #[test]
    fn test_error_conversion() {
        let fs_error = FileSystemError::DirectoryNotFound;
        let app_error: AppError = fs_error.into();
        matches!(app_error, AppError::FileSystem(_));
    }
}
