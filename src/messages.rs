//! Centralized message definitions for the Cosmic Noise application.
//!
//! This module contains all message types that flow through the application,
//! providing a single source of truth for application events and commands.

use crate::audio::AudioCommand;
use crate::errors::AppError;
use crate::models::NoiseTrack;

/// Main application message type that encompasses all possible events
#[derive(Debug, Clone)]
pub enum Message {
    /// Window and UI management messages
    Window(WindowMessage),
    /// Audio-related messages
    Audio(AudioMessage),
    /// File and track management messages
    Tracks(TrackMessage),
    /// Application lifecycle messages
    App(AppMessage),
}

/// Window and UI management messages
#[derive(Debug, Clone)]
pub enum WindowMessage {
    /// Drag window
    Drag,
    /// Maximize/restore window
    Maximize,
    /// Close window
    Close,
    /// Resize window borders
    Resize(ResizeDirection),
}

/// Window resize directions
#[derive(Debug, Clone)]
pub enum ResizeDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// Audio-related messages
#[derive(Debug, Clone)]
pub enum AudioMessage {
    /// Execute an audio command
    Command(AudioCommand),
    /// Play/pause a specific track
    PlayPause(usize),
    /// Change volume for a track
    VolumeChanged { track_id: usize, volume: f32 },
    /// Stop all playing tracks
    StopAll,
    /// Pause all playing tracks
    PauseAll,
    /// Resume all paused tracks
    ResumeAll,
    /// Set master volume
    SetMasterVolume(f32),
}

/// Track and file management messages
#[derive(Debug, Clone)]
pub enum TrackMessage {
    /// Tracks were loaded from filesystem
    Loaded(Result<Vec<NoiseTrack>, AppError>),
    /// Reload tracks from filesystem
    Reload,
    /// Track selection changed
    SelectTrack(usize),
    /// Track metadata updated
    MetadataUpdated { track_id: usize },
}

/// Application lifecycle and state messages
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Application started
    Started,
    /// Settings changed
    SettingsChanged,
    /// Error occurred
    Error(AppError),
    /// Clear current error
    ClearError,
    /// Request app statistics update
    UpdateStats,
    /// Theme changed
    ThemeChanged,
    /// Switch to settings view
    SwitchToSettings,
    /// Switch to player view
    SwitchToPlayer,

    /// Exit application
    Exit,
}

// Convenience conversions for easier message handling

impl From<WindowMessage> for Message {
    fn from(msg: WindowMessage) -> Self {
        Message::Window(msg)
    }
}

impl From<AudioMessage> for Message {
    fn from(msg: AudioMessage) -> Self {
        Message::Audio(msg)
    }
}

impl From<TrackMessage> for Message {
    fn from(msg: TrackMessage) -> Self {
        Message::Tracks(msg)
    }
}

impl From<AppMessage> for Message {
    fn from(msg: AppMessage) -> Self {
        Message::App(msg)
    }
}

impl From<AudioCommand> for AudioMessage {
    fn from(cmd: AudioCommand) -> Self {
        AudioMessage::Command(cmd)
    }
}

impl From<AudioCommand> for Message {
    fn from(cmd: AudioCommand) -> Self {
        Message::Audio(AudioMessage::Command(cmd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversions() {
        let audio_msg = AudioMessage::StopAll;
        let main_msg: Message = audio_msg.into();

        match main_msg {
            Message::Audio(AudioMessage::StopAll) => (),
            _ => panic!("Conversion failed"),
        }
    }

    #[test]
    fn test_window_message_conversion() {
        let window_msg = WindowMessage::Maximize;
        let main_msg: Message = window_msg.into();

        match main_msg {
            Message::Window(WindowMessage::Maximize) => (),
            _ => panic!("Conversion failed"),
        }
    }

    #[test]
    fn test_legacy_conversion() {
        // Removed test for LegacyMessage conversion
    }
}
