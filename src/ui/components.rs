
use crate::audio::{db_to_percentage, percentage_to_db};
use crate::errors::{AppError, AudioError, FileSystemError};
use crate::models::NoiseTrack;
use crate::ui::styles;
use crate::utils::dragwin;
use crate::utils::sine_wave_loading::SineWaveLoading;
use crate::{SPACING, fl};
use crate::audio::AudioCommand;

use iced::Alignment::Center;

use iced::widget::{
    Column, Row,  button, center_x, container, horizontal_space, row, slider, text, tooltip,
    column, 
};
use iced::{Alignment, Element, Font, Length, Theme};
use kira::sound::PlaybackState;
use std::time::Duration;

// Create a track card component
pub fn track_card(track: &NoiseTrack, index: usize) -> Element<dragwin::Message> {
    let card_content = Column::new()
        .push(track_header(track))
        .push(volume_slider(track, index))
        .push(volume_display(track))
        .spacing(SPACING)
        .width(Length::Fill)
        .height(Length::Fill);

    button(card_content)
        .style(styles::card_button_style)
        .on_press(dragwin::Message::Audio(AudioCommand::Play(index)))
        .into()
}

// Create the header section of a track card (icon + name)
fn track_header(track: &NoiseTrack) -> Row<dragwin::Message> {
    Row::new()
        .push(track_icon(track))
        .push(track_name(&track.name))
        .align_y(Alignment::Center)
}

// Create the appropriate icon based on track state
fn track_icon(track: &NoiseTrack) -> Element<dragwin::Message> {
    use iced::widget::container;
    let sine_loading = SineWaveLoading::new()
        .cycle_duration(Duration::from_secs(2))
        .radius(8.0)
        .running(matches!(track.state, PlaybackState::Playing))
        .width(50)
        .height(50);
    match track.state {
        // PlaybackState::Stopped => container(sine_loading.style(styles::loader_stopped_style)),
        PlaybackState::Paused => container(sine_loading.style(styles::loader_paused_style)),
        PlaybackState::Playing=> container(sine_loading.style(styles::loader_running_style)),

        _ => container(sine_loading.style(styles::loader_primary_style)),
    }
    .into()
}

// Create a play icon
fn play_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E805}')
}
fn close_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E803}')
}
fn minimize_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F2D1}')
}
fn pause_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E804}')
}
fn stop_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E802}')
}
fn maximize_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F2D2}')
}
fn settings_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E800}')
}
fn back_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E801}')
}

// Create an icon with the dragwin font
fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("dragwin");

    text(codepoint)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().primary.base.color),
        })
        .size(20)
        .center()
        .font(ICON_FONT)
        .into()
}
fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center_x(content).padding(0))
        .width(30)
        .height(30)
        .style(styles::card_button_style);

    if let Some(on_press) = on_press {
        tooltip(action.on_press(on_press), label, tooltip::Position::Bottom)
            .style(container::rounded_box)
            .gap(10)
            .into()
    } else {
        action.style(button::secondary).into()
    }
}
// Create a track name display
fn track_name(name: &str) -> Element<dragwin::Message> {
    text(uppercase_first(name))
        .size(14)
        .shaping(text::Shaping::Advanced)
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fill)
        .into()
}

// Create a volume slider component
pub fn volume_slider(track: &NoiseTrack, index: usize) -> Element<dragwin::Message> {
    slider(
        0.0..=100.0,
        db_to_percentage(track.volume_level),
        move |x| dragwin::Message::Audio(AudioCommand::SetVolume { track_id: index, volume: percentage_to_db(x) }),
    )
    .width(Length::Fill)
    .step(1.0)
    .height(10.0)
    .style(styles::volume_slider_style)
    .into()
}

// Create a volume percentage display
fn volume_display(track: &NoiseTrack) -> Element<dragwin::Message> {
    text(format!("{}%", db_to_percentage(track.volume_level) as u8))
        .size(10)
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fill)
        .into()
}

// Create an error display component
pub fn error_display(error: &AppError) -> Element<dragwin::Message> {
    let (icon_path, message): (&str, String) = match error {
        AppError::FileSystem(FileSystemError::DirectoryNotFound) => (
            "assets/icons/dir_not_found.svg",
            fl!("not-found"),
        ),
        AppError::FileSystem(FileSystemError::DirectoryReadError) => (
            "assets/icons/dir_not_allowed.svg",
            "Could not read audio directory. Check permissions.".to_string(),
        ),
        AppError::FileSystem(FileSystemError::InvalidFileFormat) => (
            "assets/icons/dir_not_found.svg",
            "Found an invalid or unsupported audio file format.".to_string(),
        ),
        AppError::Audio(AudioError::HandleCreationFailed) => (
            "assets/icons/dir_not_found.svg",
            fl!("pb-error"),
        ),
        _ => (
            "assets/icons/dir_not_found.svg",
            error.to_string(),
        ),
    };

    column![
        iced::widget::svg::Svg::from_path(icon_path)
            .width(200)
            .height(200)
            .style(|theme:&Theme,_st| iced::widget::svg::Style {
                color: Some(theme.extended_palette().danger.base.color), // Red color
            }),
        text(message)
            .style(styles::error_text_style)
            .size(14.0)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .wrapping(text::Wrapping::Word)
    ].align_x(iced::alignment::Horizontal::Center)
    .into()
}

// Create a toolbar component
pub fn toolbar<'a>(master_volume: f32) -> Element<'a, dragwin::Message> {
    row![
        //in this case tool bar is my button
        iced::widget::Space::new(15, 10), 
          slider(
            0.0..=100.0,
            db_to_percentage(master_volume),
            |x| dragwin::Message::Audio(AudioCommand::SetMasterVolume(percentage_to_db(x))),
        )
        .width(80)
        .step(1.0)
        .height(8)
        .style(styles::volume_slider_style),
        text(format!("{}%", db_to_percentage(master_volume) as u8))
            .size(10).style(styles::secondary_text_style)
            .align_x(iced::alignment::Horizontal::Center),
        action(play_icon(), "Resume", Some(dragwin::Message::Audio(AudioCommand::ResumeAll))),
        action(
            pause_icon(),
            text(fl!("pause-all-icon")),
            Some(dragwin::Message::Audio(AudioCommand::PauseAll)),
        ),
        action(stop_icon(), text(fl!("stop-icon")), Some(dragwin::Message::Audio(AudioCommand::StopAll))),
        iced::widget::Space::new(10, 10),
     
        horizontal_space(),
        row![
            text(fl!("app-title")).style(|t: &Theme| {
                iced::widget::text::Style {
                    color: Some(t.extended_palette().primary.base.color),
                }
            }),
           
        ]
        .align_y(Center)
        .spacing(5),
        horizontal_space(),
        action(settings_icon(), text("Settings"), Some(dragwin::Message::UI(dragwin::UIMessage::Settings))),
        action(
            minimize_icon(),
            text(fl!("minimize-icon")),
            Some(dragwin::Message::Window(dragwin::WindowMessage::Minimize))
        ),
        action(
            maximize_icon(),
            text(fl!("maximize-icon")),
            Some(dragwin::Message::Window(dragwin::WindowMessage::Maximize))
        ),
        action(close_icon(), text(fl!("close-icon")), Some(dragwin::Message::Window(dragwin::WindowMessage::Close))),
        iced::widget::Space::new(15, 10),
    ]
    .align_y(Center)
    .padding(5)
    .spacing(5)
    .into()
    //need to add button
}


// Create an empty state component when no tracks are found
pub fn empty_state<'a>() -> Element<'a, dragwin::Message> {
    container(
        Column::new()
            .push(
                text("No Audio Tracks Found")
                    .size(20)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .push(
                text("If running as a Flatpak, bundled sounds are included. To add your own sounds, place audio files in one of these directories:")
                    .size(14)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .push(
                text("• ~/.local/share/cosmic-noise/sounds/")
                    .size(12)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .push(  
                text("• ~/.config/cosmic-noise/sounds/")
                    .size(12)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .push(
                text("(Bundled sounds are in /app/share/cosmic-noise/sounds, but this is read-only in Flatpak)")
                    .size(10)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .spacing(10)
            .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

// Helper function to capitalize the first letter of a string
fn uppercase_first(data: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for value in data.chars() {
        if first {
            result.push(value.to_ascii_uppercase());
            first = false;
        } else {
            result.push(value);
        }
    }
    result
}

// Create settings view with theme selection
pub fn settings_view<'a>(current_theme: &crate::models::AppTheme) -> Element<'a, dragwin::Message> {
    use crate::models::AppTheme;
    use iced::widget::{column, pick_list, text};

    let theme_picker = pick_list(AppTheme::all(), Some(*current_theme), |theme| {
        dragwin::Message::UI(dragwin::UIMessage::ThemeChanged(theme))
    });

    let back_button = action(back_icon(), text(fl!("back")), Some(dragwin::Message::UI(dragwin::UIMessage::BackToPlayer)));

    container(
        column![
            text("Settings")
                .size(24)
                .style(styles::secondary_text_style)
                .align_x(iced::alignment::Horizontal::Center),
            row![
                text("Theme:")
                    .size(16)
                    .style(styles::secondary_text_style)
                    .align_x(iced::alignment::Horizontal::Left),
                theme_picker
            ]
            .spacing(50)
            .align_y(Center),
            back_button,
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center)
        .max_width(400),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NoiseTrack;
    use std::path::PathBuf;

    #[test]
    fn test_uppercase_first() {
        assert_eq!(uppercase_first("hello"), "Hello");
        assert_eq!(uppercase_first("HELLO"), "HELLO");
        assert_eq!(uppercase_first(""), "");
        assert_eq!(uppercase_first("a"), "A");
    }

    #[test]
    fn test_track_components() {
        let track = NoiseTrack::new("test_track".to_string(), PathBuf::from("/test/path.mp3"));

        // Test that components can be created without panicking
        let _card = track_card(&track, 0);
        let _slider = volume_slider(&track, 0);
        let _header = track_header(&track);
    }
}
