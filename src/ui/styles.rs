//! UI styling functions and theme definitions for the Cosmic Noise application.
//!
//! This module contains all styling functions and theme-related utilities
//! used throughout the application's user interface.

use iced::widget::{button, slider, text};
use iced::{Background, Border, Color, Theme};

/// Style function for track card buttons
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

/// Style function for volume sliders
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

/// Style function for error text
pub fn error_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().danger.strong.color),
    }
}

/// Style function for success text
pub fn success_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().success.strong.color),
    }
}

/// Style function for warning text
pub fn warning_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.base.text),
    }
}

/// Style function for primary text
pub fn primary_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().primary.strong.color),
    }
}

/// Style function for secondary text
pub fn secondary_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.base.text),
    }
}

/// Style function for muted text
pub fn muted_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.weak.text),
    }
}

/// Get background color for the main application
pub fn app_background_color(theme: &Theme) -> Color {
    theme.palette().background
}

/// Get primary accent color
pub fn primary_color(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.color
}

/// Get secondary accent color
pub fn secondary_color(theme: &Theme) -> Color {
    theme.extended_palette().secondary.strong.color
}

/// Get danger/error color
pub fn danger_color(theme: &Theme) -> Color {
    theme.extended_palette().danger.strong.color
}

/// Get success color
pub fn success_color(theme: &Theme) -> Color {
    theme.extended_palette().success.strong.color
}

/// Get warning color
pub fn warning_color(theme: &Theme) -> Color {
    theme.extended_palette().background.base.text
}

/// Create a rounded border with the given radius
pub fn rounded_border(radius: f32) -> Border {
    Border {
        radius: radius.into(),
        width: 1.0,
        color: Color::TRANSPARENT,
    }
}

/// Create a colored border
pub fn colored_border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: radius.into(),
    }
}
