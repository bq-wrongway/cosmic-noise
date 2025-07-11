mod app;
mod audio;
mod config;
mod errors;
mod i18n;
mod messages;
mod models;
mod ui;
mod utils;
use iced::{Color, Size, Theme, theme, window};

use crate::app::{CosmicNoise, Message};
use crate::models::AppTheme;
use crate::ui::view::main_view;

pub const SPACING: f32 = 5.0;

pub fn main() -> iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    iced::application(CosmicNoise::new, CosmicNoise::update, CosmicNoise::view)
        .font(include_bytes!("../assets/fonts/dragwin.ttf").as_slice())
        .window(window::Settings {
            transparent: true,
            decorations: false,
            size: Size::new(800., 650.),
            visible: true,

            ..Default::default()
        })
        .theme(|app: &CosmicNoise| match app.current_theme {
            AppTheme::GruvboxDark => Theme::GruvboxDark,
            AppTheme::Tokyo => Theme::TokyoNight,
            AppTheme::Catppuccin => Theme::CatppuccinMacchiato,
            AppTheme::GruvboxLight => Theme::GruvboxLight,
        })
        .style(|_, _| theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
        })
        .run()
}
impl CosmicNoise {
    fn view(&self) -> iced::Element<Message> {
        main_view(self)
    }
}
