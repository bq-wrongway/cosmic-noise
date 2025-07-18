use crate::audio::{AudioCommand, AudioSystem};
use crate::config::ConfigManager;
use crate::errors::AppError;
use crate::models::{AppTheme, NoiseTrack, View};

use crate::utils::files;
use iced::Task;
use log::info;

pub struct CosmicNoise {
    // Audio system for managing playback
    pub audio_system: AudioSystem,
    // List of available audio tracks
    pub track_list: Vec<NoiseTrack>,
    // Current error state, if any
    pub error: Option<AppError>,
    // Current view state
    pub current_view: View,
    // Current theme
    pub current_theme: AppTheme,
}

#[derive(Debug, Clone)]
pub enum Message {
    DragWin(crate::utils::dragwin::Message),
    Loaded(Result<Vec<NoiseTrack>, AppError>),
}

impl CosmicNoise {
    pub fn new() -> (Self, Task<Message>) {
        let mut audio_system = AudioSystem::new().unwrap_or_default();

        let current_theme = ConfigManager::load_theme();
        info!("Loaded theme from configuration: {current_theme:?}");

        //master volume (amplifier )
        let master_volume = ConfigManager::load_master_volume();
        audio_system.set_master_volume(master_volume);
        info!("Loaded master volume from configuration: {master_volume} dB");

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
        }
    }

    pub fn process_audio_command(&mut self, command: AudioCommand) {
        match self
            .audio_system
            .process_command(command, &mut self.track_list)
        {
            Ok(()) => {
                // Clear any previous audio errors on success
                if matches!(self.error, Some(AppError::Audio(_))) {
                    self.error = None;
                }
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
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

    #[test]
    fn test_app_creation() {
        let (app, _task) = CosmicNoise::new();
        assert!(app.track_list.is_empty());
        assert!(app.error.is_none());
    }
}
