//! Core application logic for the Cosmic Noise application.
//!
//! This module contains the main application state and business logic,
//! following the Elm architecture pattern with clear separation of concerns.

use crate::audio::{AudioCommand, AudioEvent, AudioSystem};
use crate::config::ConfigManager;
use crate::errors::AppError;
use crate::models::{AppStats, AppTheme, NoiseTrack, View};

use crate::utils::files;
use iced::Task;
use log::{error, info, warn};

/// Main application state
pub struct CosmicNoise {
    /// Audio system for managing playback
    pub audio_system: AudioSystem,
    /// List of available audio tracks
    pub track_list: Vec<NoiseTrack>,
    /// Current error state, if any
    pub error: Option<AppError>,
    /// Current view state
    pub current_view: View,
    /// Current theme
    pub current_theme: AppTheme,
}

/// Application messages that drive state changes
#[derive(Debug, Clone)]
pub enum Message {
    /// Window management messages (drag, resize, etc.)
    DragWin(crate::utils::dragwin::Message),
    /// Track loading completion
    Loaded(Result<Vec<NoiseTrack>, AppError>),
    /// Switch between views
    SwitchView(View),
    /// Change theme
    ChangeTheme(AppTheme),
}

impl CosmicNoise {
    /// Create a new application instance
    pub fn new() -> (Self, Task<Message>) {
        let audio_system = AudioSystem::new().unwrap_or_default();

        // Load theme from configuration
        let current_theme = ConfigManager::load_theme();
        info!("Loaded theme from configuration: {:?}", current_theme);

        let app = CosmicNoise {
            audio_system,
            track_list: vec![],
            error: None,
            current_view: View::default(),
            current_theme,
        };

        let task = Task::perform(files::load_data(), Message::Loaded);

        (app, task)
    }

    /// Update application state based on incoming messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragWin(drag_msg) => {
                crate::utils::dragwin::update(drag_msg, self).map(Message::DragWin)
            }
            Message::Loaded(result) => {
                match result {
                    Ok(tracks) => {
                        self.track_list = tracks;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::SwitchView(view) => {
                self.current_view = view;
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.current_theme = theme;

                // Save theme to configuration
                if let Err(e) = ConfigManager::save_theme(theme) {
                    error!("Failed to save theme to configuration: {}", e);
                    self.error = Some(e);
                } else {
                    info!("Theme saved to configuration: {:?}", theme);
                }

                Task::none()
            }
        }
    }

    /// Process audio commands and return resulting events
    pub fn process_audio_command(&mut self, command: AudioCommand) -> Vec<AudioEvent> {
        match self
            .audio_system
            .process_command(command, &mut self.track_list)
        {
            Ok(events) => {
                // Clear any previous audio errors on success
                if matches!(self.error, Some(AppError::Audio(_))) {
                    self.error = None;
                }
                events
            }
            Err(e) => {
                self.error = Some(e);
                vec![]
            }
        }
    }

    /// Get current application statistics
    pub fn get_stats(&self) -> AppStats {
        let audio_stats = self.audio_system.get_stats();
        AppStats::from_audio_stats(&audio_stats, self.track_list.len(), self.error.is_some())
    }

    /// Check if the application is in a healthy state
    pub fn is_healthy(&self) -> bool {
        self.error.is_none() && self.audio_system.is_initialized()
    }

    /// Get current error message for display
    pub fn error_message(&self) -> Option<String> {
        self.error.as_ref().map(|e| e.to_string())
    }

    /// Clear current error state
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Reload tracks from the file system
    pub fn reload_tracks(&mut self) -> Task<Message> {
        Task::perform(files::load_data(), Message::Loaded)
    }

    /// Get track by index safely
    pub fn get_track(&self, index: usize) -> Option<&NoiseTrack> {
        self.track_list.get(index)
    }

    /// Get mutable track by index safely
    pub fn get_track_mut(&mut self, index: usize) -> Option<&mut NoiseTrack> {
        self.track_list.get_mut(index)
    }

    /// Get all tracks
    pub fn tracks(&self) -> &[NoiseTrack] {
        &self.track_list
    }

    /// Check if any tracks are currently playing
    pub fn has_playing_tracks(&self) -> bool {
        self.audio_system.get_stats().playing_tracks > 0
    }

    /// Stop all currently playing tracks
    pub fn stop_all(&mut self) -> Vec<AudioEvent> {
        self.process_audio_command(AudioCommand::StopAll)
    }

    /// Pause all currently playing tracks
    pub fn pause_all(&mut self) -> Vec<AudioEvent> {
        self.process_audio_command(AudioCommand::PauseAll)
    }

    /// Resume all paused tracks
    pub fn resume_all(&mut self) -> Vec<AudioEvent> {
        self.process_audio_command(AudioCommand::ResumeAll)
    }
}

impl Default for CosmicNoise {
    fn default() -> Self {
        Self {
            audio_system: AudioSystem::default(),
            track_list: vec![],
            error: None,
            current_view: View::default(),
            current_theme: ConfigManager::load_theme(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_app_creation() {
        let (app, _task) = CosmicNoise::new();
        assert!(app.track_list.is_empty());
        assert!(app.error.is_none());
    }

    #[test]
    fn test_app_stats() {
        let (app, _) = CosmicNoise::new();
        let stats = app.get_stats();
        assert_eq!(stats.total_tracks, 0);
        assert_eq!(stats.loaded_tracks, 0);
        assert_eq!(stats.playing_tracks, 0);
        assert_eq!(stats.paused_tracks, 0);
    }

    #[test]
    fn test_app_health() {
        let (app, _) = CosmicNoise::new();
        // App should be healthy initially (no errors)
        // Audio initialization might fail in test environment, so we only check error state
        assert!(app.error.is_none());
    }

    #[test]
    fn test_app_stats_status_message() {
        let stats = AppStats {
            total_tracks: 5,
            loaded_tracks: 5,
            playing_tracks: 2,
            paused_tracks: 0,
            has_error: false,
            audio_initialized: true,
            uptime: Duration::from_secs(0),
            memory_usage_mb: None,
        };
        assert_eq!(stats.status_message(), "Playing 2 tracks");

        let stats = AppStats {
            total_tracks: 0,
            loaded_tracks: 0,
            playing_tracks: 0,
            paused_tracks: 0,
            has_error: false,
            audio_initialized: true,
            uptime: Duration::from_secs(0),
            memory_usage_mb: None,
        };
        assert_eq!(stats.status_message(), "No tracks found");
    }
}
