// SPDX-License-Identifier: GPL-3.0-only
use crate::files::{self, NoiseTrack};
use crate::fl;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Limits};
use cosmic::iced_runtime::window::Id;
use cosmic::iced_sctk::commands::popup::{destroy_popup, get_popup};
use cosmic::iced_style::application;
use cosmic::iced_widget::scrollable;
use cosmic::widget::{container, flex_row, horizontal_space, mouse_area, slider, Column, Row};
use cosmic::{widget, Application, Element, Theme};
use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings},
        FromFileError, PlaybackState,
    },
    tween::{Easing, Tween},
    StartTime,
};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
const PADDING: f32 = 20.0;
const SPACING: f32 = 10.0;
const MIN_WIDTH: f32 = 200.0;
const MIN_HEIGHT: f32 = 100.0;
const LINEAR_TWEEN: Tween = Tween {
    duration: Duration::from_secs(1),
    easing: Easing::Linear,
    start_time: StartTime::Immediate,
};

const ID: &str = "io.github.bq-wrongway.CosmicNoise";
/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
// #[derive(Clone, Default)]
pub struct YourApp {
    /// This is the core of your application, it is used to communicate with the Cosmic runtime.
    /// It is used to send messages to your application, and to access the resources of the Cosmic runtime.
    core: Core,
    popup: Option<Id>,
    manager: AudioManager,
    files: Vec<NoiseTrack>,
    currently_playing: HashMap<usize, StreamingSoundHandle<FromFileError>>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Play(usize),
    VolumeChanged((f32, usize)),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the executor that will be used to run your application.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for YourApp {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.wrongway.CosmicNoiseApplet";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the header of your application, it can be used to display the title of your application.
    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![widget::text::heading(fl!("app-title")).into()]
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let example = YourApp {
            core,
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .ok()
                .unwrap(),
            files: files::load_data(),
            currently_playing: HashMap::new(),
            popup: None,
            // ..Default::default()
        };

        (example, Command::none())
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> cosmic::iced::Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::Play(i) => match self.currently_playing.get_mut(&i) {
                Some(h) => match h.state() {
                    PlaybackState::Playing => {
                        self.files.get_mut(i).unwrap().is_playing = false;
                        h.pause(LINEAR_TWEEN);
                    }

                    _ => {
                        self.files.get_mut(i).unwrap().is_playing = true;
                        h.resume(Tween::default())
                    }
                },
                None => {
                    let settings = StreamingSoundSettings::new().loop_region(0.0..);
                    let sound_data =
                        StreamingSoundData::from_file(Path::new(&self.files[i].path)).unwrap();

                    let handler = self
                        .manager
                        .play(sound_data.with_settings(settings))
                        .unwrap();
                    self.currently_playing.insert(i, handler);
                    self.files.get_mut(i).unwrap().is_playing = true;
                }
            },
            Message::VolumeChanged(level) => {
                println!("{:?}", level);
                let (f, s) = level;

                match self.currently_playing.get_mut(&s) {
                    Some(t) => {
                        t.set_volume(f as f64, Tween::default());
                        self.files.get_mut(s).unwrap().volume_level = f;
                    }
                    None => {
                        println!("asd");
                    }
                }
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = cosmic::iced_runtime::window::Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(480.0)
                        .min_width(400.0)
                        .min_height(200.0)
                        .max_height(325.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Command::none()
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let content = flex_row(get_elements(&self.files));
        let main_cot = scrollable(container(content).width(500.0).height(400.0).padding(10.0))
            .width(480.0)
            .height(400.);

        self.core
            .applet
            .popup_container(main_cot)
            .width(Length::Fixed(480.))
            .height(Length::Fixed(400.))
            .into()
    }
    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
fn get_component(t: &NoiseTrack, i: usize) -> Column<Message> {
    cosmic::widget::column()
        .push(
            cosmic::widget::row()
                .push(
                    cosmic::iced::widget::text(uppercase_first(&t.name))
                        .style(cosmic::style::Text::Accent)
                        .size(12)
                        .shaping(cosmic::iced_widget::text::Shaping::Advanced)
                        .height(Length::Fill)
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                )
                // .push(cosmic::iced::widget::text("*"))
                .align_items(cosmic::iced_core::Alignment::Center),
        )
        .push(
            slider(0.0..=4.0, t.volume_level, move |x| {
                Message::VolumeChanged((x, i))
            })
            .width(Length::Fill)
            .step(0.01)
            .height(10.0),
        )
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill)
}
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

fn get_elements(files: &Vec<NoiseTrack>) -> Vec<Element<Message>> {
    let mut new_vec = vec![];
    for (i, t) in files.iter().enumerate() {
        new_vec.push(
            mouse_area(
                container(get_component(&t, i))
                    .width(150.0)
                    .height(75.0)
                    .style(if t.is_playing {
                        cosmic::style::iced::Container::Secondary
                    } else {
                        cosmic::style::iced::Container::Primary
                    })
                    .padding(4.),
            )
            .on_press(Message::Play(i))
            .into(),
        )
    }
    new_vec
}
