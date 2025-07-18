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

// Window management messages for drag, resize, maximize, minimize, close
#[derive(Debug, Clone)]
pub enum WindowMessage {
    Drag,
    Maximize,
    Minimize,
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    South,
    SouthWest,
    SouthEast,
    Close,
}

// UI navigation messages for settings, navigation, theme changes
#[derive(Debug, Clone)]
pub enum UIMessage {
    Settings,
    BackToPlayer,
    ThemeChanged(crate::models::AppTheme),
}

// Combined message type that can handle all three message types
#[derive(Debug, Clone)]
pub enum Message {
    Window(WindowMessage),
    Audio(AudioCommand),
    UI(UIMessage),
}

pub fn update(message: Message, cnoise: &mut CosmicNoise) -> Task<Message> {
    match message {
        Message::Window(window_msg) => match window_msg {
            WindowMessage::Drag => window::get_latest()
                .and_then(window::drag)
                .map(Message::Window),
            WindowMessage::Maximize => {
                println!("toggle!");
                window::get_latest()
                    .and_then(window::toggle_maximize)
                    .map(Message::Window)
            }
            WindowMessage::Minimize => window::get_latest()
                .and_then(|id| window::minimize(id, true))
                .map(Message::Window),
            WindowMessage::NorthWest => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::NorthWest))
                .map(Message::Window),
            WindowMessage::North => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::North))
                .map(Message::Window),
            WindowMessage::NorthEast => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::NorthEast))
                .map(Message::Window),
            WindowMessage::West => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::West))
                .map(Message::Window),
            WindowMessage::East => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::East))
                .map(Message::Window),
            WindowMessage::South => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::South))
                .map(Message::Window),
            WindowMessage::SouthWest => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::SouthWest))
                .map(Message::Window),
            WindowMessage::SouthEast => window::get_latest()
                .and_then(|f| drag_resize(f, window::Direction::SouthEast))
                .map(Message::Window),
            WindowMessage::Close => window::get_latest()
                .and_then(window::close)
                .map(Message::Window),
        },
        Message::Audio(audio_cmd) => {
            cnoise.process_audio_command(audio_cmd);
            Task::none()
        }
        Message::UI(ui_msg) => {
            match ui_msg {
                UIMessage::Settings => {
                    // Switch to settings view
                    cnoise.current_view = crate::models::View::Settings;
                }
                UIMessage::BackToPlayer => {
                    // Switch back to player view
                    cnoise.current_view = crate::models::View::Player;
                }
                UIMessage::ThemeChanged(theme) => {
                    // Update theme in app state
                    cnoise.current_theme = theme;

                    // Save theme to configuration
                    if let Err(e) = crate::config::ConfigManager::save_theme(theme) {
                        log::error!("Failed to save theme to configuration: {e}");
                        cnoise.error = Some(e);
                    } else {
                        log::info!("Theme saved to configuration: {theme}");
                    }
                }
            }
            Task::none()
        }
    }
}

pub fn view<'a>(content: Element<'a, Message>, cnoise: &CosmicNoise) -> Element<'a, Message> {
    let master_volume = cnoise.audio_system.master_volume();

    let base = iced::widget::container(
        iced::widget::column![
            mouse_area(
                iced::widget::container(toolbar(master_volume))
                    .align_y(Center)
                    .width(Fill)
                    .height(40)
            )
            .on_double_click(Message::Window(WindowMessage::Maximize))
            .on_press(Message::Window(WindowMessage::Drag)),
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
        .on_press(Message::Window(WindowMessage::SouthWest))
        .interaction(Interaction::ResizingDiagonallyUp),
        mouse_area(
            iced::widget::container(row![])
                .width(Fill)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::Window(WindowMessage::South))
        .interaction(Interaction::ResizingVertically),
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::Window(WindowMessage::SouthEast))
        .interaction(Interaction::ResizingDiagonallyDown),
    ];

    let top_row = row![
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::Window(WindowMessage::NorthWest))
        .interaction(Interaction::ResizingDiagonallyDown),
        mouse_area(
            iced::widget::container(row![])
                .width(Fill)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::Window(WindowMessage::North))
        .interaction(Interaction::ResizingVertically),
        mouse_area(
            iced::widget::container(row![])
                .width(5)
                .height(2)
                .style(|_| border_container())
        )
        .on_press(Message::Window(WindowMessage::NorthEast))
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
                .on_press(Message::Window(WindowMessage::West))
                .interaction(Interaction::ResizingHorizontally),
                base,
                mouse_area(
                    iced::widget::container(row![])
                        .width(2)
                        .height(Fill)
                        .style(|_| border_container())
                )
                .on_press(Message::Window(WindowMessage::East))
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
