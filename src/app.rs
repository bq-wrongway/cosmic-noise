// SPDX-License-Identifier: GPL-3.0-only
use crate::files::{self, NoiseTrack};
use crate::fl;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::iced_widget::scrollable;
use cosmic::widget::{container, flex_row, mouse_area, slider, Column};
use cosmic::{widget, Application, Element};
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

const LINEAR_TWEEN: Tween = Tween {
    duration: Duration::from_secs(1),
    easing: Easing::Linear,
    start_time: StartTime::Immediate,
};

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
// #[derive(Clone, Default)]
pub struct CosmicNoise {
    /// This is the core of your application, it is used to communicate with the Cosmic runtime.
    /// It is used to send messages to your application, and to access the resources of the Cosmic runtime.
    core: Core,
    manager: AudioManager,
    files: Vec<NoiseTrack>,
    currently_playing: HashMap<usize, StreamingSoundHandle<FromFileError>>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
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
impl Application for CosmicNoise {
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
        let example = CosmicNoise {
            core,
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .ok()
                .unwrap(),
            files: files::load_data(),
            currently_playing: HashMap::new(),
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
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = flex_row(get_elements(&self.files));
        let main_cot = scrollable(container(content).padding(10.0));
        main_cot.into()
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
