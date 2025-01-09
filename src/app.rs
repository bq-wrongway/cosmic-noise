use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Limits};
use cosmic::iced_core::Padding;
use cosmic::iced_runtime::window::Id;
use cosmic::iced_sctk::commands::popup::{destroy_popup, get_popup};
use cosmic::iced_style::application;
use cosmic::iced_widget::{row, scrollable, text};
use cosmic::widget::{container, flex_row, horizontal_space, mouse_area, slider, Column, Row};
use cosmic::{widget, Application, Element, Theme};
use kira::{
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings},
        FromFileError, PlaybackState,
    },
    StartTime,
    {backend::DefaultBackend, AudioManager, AudioManagerSettings},
};
use kira::{Easing, Tween};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

// SPDX-License-Identifier: GPL-3.0-only
use crate::files::{self, NoiseTrack};
use crate::fl;

const PADDING: f32 = 20.0;
const SPACING: f32 = 10.0;
const MIN_WIDTH: f32 = 200.0;
const MIN_HEIGHT: f32 = 100.0;
const LINEAR_TWEEN: Tween = Tween {
    duration: Duration::from_secs(1),
    easing: Easing::Linear,
    start_time: StartTime::Immediate,
};

// #[derive(Clone, Default)]
pub struct CosmicNoise {
    core: Core,
    popup: Option<Id>,
    manager: AudioManager,
    files: Vec<NoiseTrack>,
    currently_playing: HashMap<usize, StreamingSoundHandle<FromFileError>>,
    state: PlaybackState,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Play(usize),
    VolumeChanged((f32, usize)),
    StopAll,
    PauseAll,
    ResumeAll,
}
impl Application for CosmicNoise {
    type Executor = cosmic::executor::Default;
    const APP_ID: &'static str = "io.github.bq-wrongway.CosmicNoise";

    type Flags = ();

    type Message = Message;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![widget::text::heading(fl!("app-title")).into()]
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let cosmic_noise = CosmicNoise {
            core,
            popup: None,
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .ok()
                .unwrap(),
            files: files::load_data(),
            currently_playing: HashMap::new(),
            state: PlaybackState::Stopped,
            error: None,
            // ..Default::default()batch
        };

        (cosmic_noise, Command::none())
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> cosmic::iced::Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::Play(i) => match self.currently_playing.get_mut(&i) {
                Some(h) => match h.state() {
                    PlaybackState::Playing => {
                        h.pause(LINEAR_TWEEN);
                        let cur_p = self.files.get_mut(i).unwrap();
                        cur_p.state = PlaybackState::Paused;
                    }

                    PlaybackState::Paused => {
                        h.resume(Tween::default());
                        self.files.get_mut(i).unwrap().state = PlaybackState::Playing;
                    }
                    _ => {
                        // h.resume(Tween::default());
                        self.files.get_mut(i).unwrap().state = PlaybackState::Stopped;
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
                    self.files.get_mut(i).unwrap().state = PlaybackState::Playing;
                }
            },
            Message::VolumeChanged(level) => {
                println!("{:?}", level);
                let (f, s) = level;

                match self.currently_playing.get_mut(&s) {
                    Some(t) => {
                        t.set_volume(f, Tween::default());
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
            Message::StopAll => {
                if !&self.currently_playing.is_empty() {
                    for (n, t) in &mut self.currently_playing {
                        t.stop(Tween::default());
                        self.files.get_mut(*n).unwrap().state = PlaybackState::Stopped;
                    }
                }
                self.currently_playing.clear();
                self.state = PlaybackState::Stopped;

                println!("{:?}", self.currently_playing.is_empty());
            }
            Message::PauseAll => {
                if !&self.currently_playing.is_empty() {
                    self.currently_playing.iter_mut().for_each(|(_n, t)| {
                        t.pause(Tween::default());
                        // self.files.get_mut(*n).unwrap().is_playing = false;
                    });
                    self.state = PlaybackState::Paused;
                }
                println!("{:?}", self.currently_playing.is_empty());
            }
            Message::ResumeAll => {
                if !&self.currently_playing.is_empty() {
                    self.currently_playing.iter_mut().for_each(|(_n, t)| {
                        t.resume(Tween::default());
                        // self.files.get_mut(*n).unwrap().is_playing = false;
                    });
                    self.state = PlaybackState::Playing;
                }
                println!("{:?}", self.currently_playing.is_empty());
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("io.github.bqwrongway.wave-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        //need to pay attention to flex row, since its inside of scrollable it might need to be wrapped by the container (no width/noheight settigns)
        let content = flex_row(get_elements(&self.files));

        let play_pause = row![
            mouse_area(
                container(cosmic::widget::icon::from_name(
                    "io.github.bqwrongway.pause-symbolic",
                ))
                .width(20)
                .padding(0)
                .style(cosmic::style::Container::Transparent)
                .height(20),
            )
            .on_press(Message::PauseAll),
            mouse_area(
                container(cosmic::widget::icon::from_name(
                    "io.github.bqwrongway.play-symbolic",
                ))
                .width(20)
                .padding(0)
                .style(cosmic::style::Container::Transparent)
                .height(20),
            )
            .on_press(Message::ResumeAll)
        ];
        let nav_row = Row::new()
            .push(
                mouse_area(
                    container(cosmic::widget::icon::from_name(
                        "io.github.bqwrongway.stop-symbolic",
                    ))
                    .width(20)
                    .padding(0)
                    .style(cosmic::style::Container::Transparent)
                    .height(20),
                )
                .on_press(Message::StopAll),
            )
            .push(horizontal_space(Length::Fill))
            .push(text("CosmicNoise"))
            .push(horizontal_space(Length::Fill))
            .push(play_pause)
            .width(500.0)
            .height(Length::Shrink)
            .align_items(Alignment::Center);
        let main_content = Column::new()
            .push(nav_row)
            .push(
                container(scrollable(container(content).padding(Padding {
                    top: 0.,
                    right: 10.,
                    bottom: 5.,
                    left: 0.,
                })))
                .height(320)
                .width(500),
            )
            .spacing(5)
            .width(480.0)
            .height(400.)
            .padding(5);

        self.core
            .applet
            .popup_container(main_content)
            .width(Length::Fixed(480.))
            .height(Length::Fixed(400.))
            .into()
    }
    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

//need to deal with styling and global pause  resume
fn get_component(t: &NoiseTrack, i: usize) -> Column<Message> {
    cosmic::widget::column()
        .push(
            cosmic::widget::row()
                .push(
                    cosmic::iced::widget::text(uppercase_first(&t.name))
                        .style(match t.state {
                            PlaybackState::Paused => cosmic::style::Text::Default,
                            _ => cosmic::style::Text::Accent,
                        })
                        .size(12)
                        .shaping(cosmic::iced_widget::text::Shaping::Advanced)
                        .height(Length::Fill)
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                )
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
//need to deal with styling and global pause  resume

//get VIEW elements to be presented
fn get_elements(files: &[NoiseTrack]) -> Vec<Element<Message>> {
    let mut new_vec = vec![];
    for (i, t) in files.iter().enumerate() {
        new_vec.push(
            mouse_area(
                container(get_component(t, i))
                    .width(150.0)
                    .height(75.0)
                    .style(match t.state {
                        PlaybackState::Playing => cosmic::style::iced::Container::Secondary,
                        _ => cosmic::style::iced::Container::Primary,
                    })
                    .padding(4.),
            )
            .on_press(Message::Play(i))
            .into(),
        )
    }
    new_vec
}
//first letter set to uppercase
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

pub enum Error {
    FileSystem,
    PlayBack,
}
