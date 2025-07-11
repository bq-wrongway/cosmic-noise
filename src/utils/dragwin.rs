use iced::{
    Alignment::Center,
    Background, Color, Element,
    Length::Fill,
    Task, Theme,
    mouse::Interaction,
    widget::{
        container::{self, Style},
        mouse_area, row,
    },
    window::{self, drag_resize},
};

use crate::{CosmicNoise, audio::AudioCommand, ui::components::toolbar};

#[derive(Debug, Clone)]
pub enum Message {
    Drag,
    Maximize,
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    South,
    SouthWest,
    SouthEast,
    Close,
    Play(usize),
    VolumeChanged((f32, usize)),
    MasterVolumeChanged(f32),
    StopAll,
    PauseAll,
    ResumeAll,
    Settings,
    BackToPlayer,
    ThemeChanged(crate::models::AppTheme),
}

pub fn update(message: Message, cnoise: &mut CosmicNoise) -> Task<Message> {
    match message {
        Message::Drag => window::get_latest().and_then(window::drag),
        Message::Maximize => {
            println!("toggle!");
            // Task::none()
            window::get_latest().and_then(window::toggle_maximize)
        }
        Message::NorthWest => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::NorthWest))
        }
        Message::North => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::North))
        }
        Message::NorthEast => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::NorthEast))
        }
        Message::West => window::get_latest().and_then(|f| drag_resize(f, window::Direction::West)),
        Message::East => window::get_latest().and_then(|f| drag_resize(f, window::Direction::East)),
        Message::South => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::South))
        }
        Message::SouthWest => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::SouthWest))
        }
        Message::SouthEast => {
            window::get_latest().and_then(|f| drag_resize(f, window::Direction::SouthEast))
        }
        Message::Close => window::get_latest().and_then(window::close),

        Message::Play(i) => {
            cnoise.process_audio_command(AudioCommand::Play(i));
            Task::none()
        }
        Message::VolumeChanged(level) => {
            let (volume, track_id) = level;
            cnoise.process_audio_command(AudioCommand::SetVolume { track_id, volume });
            Task::none()
        }
        Message::MasterVolumeChanged(volume) => {
            cnoise.process_audio_command(AudioCommand::SetMasterVolume(volume));
            Task::none()
        }
        Message::StopAll => {
            cnoise.process_audio_command(AudioCommand::StopAll);
            Task::none()
        }
        Message::PauseAll => {
            cnoise.process_audio_command(AudioCommand::PauseAll);
            Task::none()
        }
        Message::ResumeAll => {
            cnoise.process_audio_command(AudioCommand::ResumeAll);
            Task::none()
        }
        Message::Settings => {
            // Switch to settings view
            cnoise.current_view = crate::models::View::Settings;
            Task::none()
        }
        Message::BackToPlayer => {
            // Switch back to player view
            cnoise.current_view = crate::models::View::Player;
            Task::none()
        }
        Message::ThemeChanged(theme) => {
            // Update theme in app state
            cnoise.current_theme = theme;

            // Save theme to configuration
            if let Err(e) = crate::config::ConfigManager::save_theme(theme) {
                log::error!("Failed to save theme to configuration: {}", e);
                cnoise.error = Some(e);
            } else {
                log::info!("Theme saved to configuration: {:?}", theme);
            }

            Task::none()
        }
    }
}

pub fn view<'a>(
    content: Element<'a, Message>,
    cnoise: &CosmicNoise,
    //doing this also does not work
    // toolbar: Element<'a, crate::Message>,
) -> Element<'a, Message> {
    let master_volume = cnoise.audio_system.master_volume();
    
    let base = iced::widget::container(
        iced::widget::column![
            mouse_area(
                iced::widget::container(toolbar(master_volume))
                    .align_y(Center)
                    .width(Fill)
                    .height(40)
            )
            .on_double_click(Message::Maximize)
            .on_press(Message::Drag),
        ]
        .push(content),
    )
    .style(|t: &Theme| Style {
        background: Some(Background::Color(t.palette().background)),
        border: iced::Border {
            color: t.palette().warning,
            width: 1.,
            radius: 8.into(),
        },
        ..Default::default()
    })
    .align_x(Center)
    .center_x(Fill)
    .width(Fill)
    .height(Fill);
    let bottom_row = row![
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::SouthWest)
        .interaction(Interaction::ResizingDiagonallyUp),
        mouse_area(
            iced::widget::container(row![])
                .width(Fill)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::South)
        .interaction(Interaction::ResizingVertically),
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::SouthEast)
        .interaction(Interaction::ResizingDiagonallyDown),
    ];

    let top_row = row![
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::NorthWest)
        .interaction(Interaction::ResizingDiagonallyDown),
        mouse_area(
            iced::widget::container(row![])
                .width(Fill)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::North)
        .interaction(Interaction::ResizingVertically),
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::NorthEast)
        .interaction(Interaction::ResizingDiagonallyUp),
    ];

    let rebase: Element<_> = iced::widget::center(iced::widget::column![
        top_row,
        iced::widget::container(
            row![
                mouse_area(
                    iced::widget::container(row![])
                        .width(2)
                        .height(Fill)
                        .style(|_| border_container())
                )
                .on_press(Message::West)
                .interaction(Interaction::ResizingHorizontally),
                base,
                mouse_area(
                    iced::widget::container(row![])
                        .width(2)
                        .height(Fill)
                        .style(|_| border_container())
                )
                .on_press(Message::East)
                .interaction(Interaction::ResizingHorizontally),
            ]
            .width(Fill)
            .height(Fill)
        )
        .width(Fill)
        .height(Fill),
        bottom_row
    ])
    .align_x(Center)
    .center_x(Fill)
    .width(Fill)
    .height(Fill)
    .into();
    rebase
}
fn border_container() -> Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    }
}
