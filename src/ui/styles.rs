use iced::widget::{button, slider, text};
use iced::{Background, Border, Color, Theme};

use crate::utils::sine_wave_loading;

// Style function for track card buttons
pub fn card_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let bg_color = palette.background.strongest.color;
    let hover_bg_color = palette.background.base.color;
    let text_color = palette.background.strongest.text;
    let weak_text = palette.background.base.text;

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(bg_color)),
            text_color,
            border: Border {
                color: text_color.inverse(),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(hover_bg_color)),
            text_color: weak_text,
            border: Border {
                color: weak_text.inverse(),
                width: 2.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                color: palette.background.weak.text,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(hover_bg_color)),
            text_color: weak_text,
            border: Border {
                color: weak_text.inverse(),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    }
}

pub fn loader_running_style(theme: &Theme) -> sine_wave_loading::Style {
    let palette = theme.extended_palette();
    sine_wave_loading::Style {
        color: palette.success.strong.color,
        background_color: Color::TRANSPARENT,
    }
}
pub fn loader_paused_style(theme: &Theme) -> sine_wave_loading::Style {
    let palette = theme.extended_palette();
    sine_wave_loading::Style {
        color: palette.warning.strong.color,
        background_color: Color::TRANSPARENT,
    }
}
pub fn loader_primary_style(theme: &Theme) -> sine_wave_loading::Style {
    let palette = theme.extended_palette();
    sine_wave_loading::Style {
        color: palette.primary.strong.color,
        background_color: Color::TRANSPARENT,
    }
}

// Style function for volume sliders
pub fn volume_slider_style(theme: &Theme, _status: slider::Status) -> slider::Style {
    let palette = theme.extended_palette();

    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(palette.primary.base.color),
                Background::Color(palette.primary.strong.color),
            ),
            width: 4.0,
            border: Border {
                radius: 2.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(palette.primary.strong.color),
            border_color: palette.primary.strong.text,
            border_width: 1.0,
        },
    }
}

// Style function for error text
pub fn error_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().danger.strong.color),
    }
}

// Style function for secondary text
pub fn secondary_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.base.text),
    }
}
