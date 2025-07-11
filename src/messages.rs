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

// Legacy message type for backward compatibility during refactoring
#[derive(Debug, Clone)]
pub enum LegacyMessage {
    DragWin(crate::utils::dragwin::Message),
    Loaded(Result<Vec<NoiseTrack>, AppError>),
}

impl From<LegacyMessage> for Message {
    fn from(legacy: LegacyMessage) -> Self {
        match legacy {
            LegacyMessage::DragWin(drag_msg) => {
                // Convert dragwin messages to appropriate window messages
                match drag_msg {
                    crate::utils::dragwin::Message::Drag => Message::Window(WindowMessage::Drag),
                    crate::utils::dragwin::Message::Maximize => {
                        Message::Window(WindowMessage::Maximize)
                    }
                    crate::utils::dragwin::Message::Close => Message::Window(WindowMessage::Close),
                    crate::utils::dragwin::Message::North => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::North))
                    }
                    crate::utils::dragwin::Message::NorthEast => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::NorthEast))
                    }
                    crate::utils::dragwin::Message::East => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::East))
                    }
                    crate::utils::dragwin::Message::SouthEast => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::SouthEast))
                    }
                    crate::utils::dragwin::Message::South => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::South))
                    }
                    crate::utils::dragwin::Message::SouthWest => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::SouthWest))
                    }
                    crate::utils::dragwin::Message::West => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::West))
                    }
                    crate::utils::dragwin::Message::NorthWest => {
                        Message::Window(WindowMessage::Resize(ResizeDirection::NorthWest))
                    }
                    crate::utils::dragwin::Message::Play(i) => {
                        Message::Audio(AudioMessage::PlayPause(i))
                    }
                    crate::utils::dragwin::Message::VolumeChanged((volume, track_id)) => {
                        Message::Audio(AudioMessage::VolumeChanged { track_id, volume })
                    }
                    crate::utils::dragwin::Message::StopAll => {
                        Message::Audio(AudioMessage::StopAll)
                    }
                    crate::utils::dragwin::Message::PauseAll => {
                        Message::Audio(AudioMessage::PauseAll)
                    }
                    crate::utils::dragwin::Message::ResumeAll => {
                        Message::Audio(AudioMessage::ResumeAll)
                    }
                    crate::utils::dragwin::Message::Settings => {
                        Message::App(AppMessage::SwitchToSettings)
                    }
                    crate::utils::dragwin::Message::BackToPlayer => {
                        Message::App(AppMessage::SwitchToPlayer)
                    }
                    crate::utils::dragwin::Message::ThemeChanged(theme) => {
                        Message::App(AppMessage::ThemeChanged)
                    }
                }
            }
            LegacyMessage::Loaded(result) => Message::Tracks(TrackMessage::Loaded(result)),
        }
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
        let legacy = LegacyMessage::DragWin(crate::utils::dragwin::Message::StopAll);
        let main_msg: Message = legacy.into();

        match main_msg {
            Message::Audio(AudioMessage::StopAll) => (),
            _ => panic!("Legacy conversion failed"),
        }
    }
}
