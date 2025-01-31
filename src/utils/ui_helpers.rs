use cosmic::iced::Border;
use cosmic::iced_widget::container;
use cosmic::style;

//container helper
//need to add color
pub fn paused_contaner<'a>() -> style::Container<'a> {
    cosmic::style::Container::custom(|t| container::Style {
        text_color: Some(t.cosmic().warning_color().into()),
        background: Some(cosmic::iced::Background::Color(
            t.cosmic().secondary_container_color().into(),
        )),
        border: Border {
            color: t.cosmic().warning_color().into(),
            width: 1.0,
            radius: 8.into(),
        },
        ..Default::default()
    })
}
pub fn playing_contaner<'a>() -> style::Container<'a> {
    cosmic::style::Container::custom(|t| container::Style {
        text_color: Some(t.cosmic().success_color().into()),
        border: Border {
            color: t.cosmic().success_color().into(),
            width: 1.0,
            radius: 8.into(),
        },
        background: Some(cosmic::iced::Background::Color(
            t.cosmic().secondary_container_color().into(),
        )),
        ..Default::default()
    })
}
pub fn idle_container<'a>() -> style::Container<'a> {
    cosmic::style::Container::custom(|t| container::Style {
        background: Some(cosmic::iced::Background::Color(
            t.cosmic().bg_color().into(),
        )),

        ..Default::default()
    })
}
