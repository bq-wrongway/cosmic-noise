//! Main view module for the Cosmic Noise application.
//!
//! This module contains the main view function that renders the entire application
//! interface. It orchestrates all the components and handles the overall layout.

use crate::app::{CosmicNoise, Message};
use crate::models::{NoiseTrack, View};
use crate::ui::components::{empty_state, error_display, settings_view, spacer, track_card};
use crate::utils::dragwin;

use iced::Element;
use iced::widget::{column,center, container, grid, row, scrollable};

/// Main view function that renders the entire application
pub fn main_view(app: &CosmicNoise) -> Element<Message> {
    let main_content = column![content_area(app)].padding(10);

    dragwin::view(main_content.into(), app).map(Message::DragWin)
}

/// Create the main content area
fn content_area(app: &CosmicNoise) -> Element<dragwin::Message> {
    match app.current_view {
        View::Player => {
            // Show error if present
            if let Some(error) = &app.error {
                return center(error_display(error)).into();
            }
            // Show empty state if no tracks
            if app.track_list.is_empty() {
                return empty_state();
            }

            // Show tracks grid
            tracks_grid(&app.track_list)
        }
        View::Settings => settings_view(&app.current_theme),
    }
}

/// Create a scrollable grid of track cards
fn tracks_grid(tracks: &[NoiseTrack]) -> Element<dragwin::Message> {
    let track_elements: Vec<Element<dragwin::Message>> = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| track_card(track, index))
        .collect();

    container(scrollable(
        row![
            grid(track_elements)
                .spacing(5)
                .height(iced::widget::grid::aspect_ratio(200, 150))
                .fluid(210)
        ]
        .push(spacer(18, 1)),
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioSystem;
    use crate::models::NoiseTrack;
    use std::path::PathBuf;

    fn create_test_app() -> CosmicNoise {
        CosmicNoise {
            audio_system: AudioSystem::default(),
            track_list: vec![],
            error: None,
            current_view: View::default(),
            current_theme: crate::config::ConfigManager::load_theme(),
        }
    }

    #[test]
    fn test_main_view_empty() {
        let app = create_test_app();
        let _view = main_view(&app);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_main_view_with_tracks() {
        let mut app = create_test_app();
        app.track_list = vec![
            NoiseTrack::new("track1".to_string(), PathBuf::from("/test/track1.mp3")),
            NoiseTrack::new("track2".to_string(), PathBuf::from("/test/track2.mp3")),
        ];
        let _view = main_view(&app);
        // Test passes if no panic occurs
    }
}
