use crate::fl;
use cosmic::app::Core;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Alignment::Center;
use cosmic::iced::{Alignment, Border, Length, Limits, Pixels};
use cosmic::iced_core::window::Id;
use cosmic::iced_core::Padding;
use cosmic::iced_widget::text::Shaping::Advanced;
use cosmic::iced_widget::{horizontal_rule, row, scrollable, text};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{
    container, flex_row, horizontal_space, mouse_area, slider, Column, Row, Space,
};
use cosmic::{style, widget, Application, Element, Task};
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
use std::time::Duration;

// SPDX-License-Identifier: GPL-3.0-only
use crate::files::{self, NoiseTrack};

const SPACING: f32 = 5.0;
const MAX_WIDTH: f32 = 150.0;
const MAX_HEIGHT: f32 = 75.0;
const LINEAR_TWEEN: Tween = Tween {
    duration: Duration::from_secs(1),
    easing: Easing::Linear,
    start_time: StartTime::Immediate,
};

// need to introduce proper error handling
// #[derive(Clone, Default)]
pub struct CosmicNoise {
    core: Core,
    popup: Option<Id>,
    manager: Option<AudioManager>,
    track_list: Vec<NoiseTrack>,
    currently_playing: HashMap<usize, StreamingSoundHandle<FromFileError>>,
    state: PlaybackState,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    // PopupClosed(Id),
    Play(usize),
    VolumeChanged((f32, usize)),
    StopAll,
    PauseAll,
    ResumeAll,
}
impl Application for CosmicNoise {
    type Executor = cosmic::executor::Default;
    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "io.github.bq-wrongway.CosmicNoise";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (CosmicNoise, cosmic::Task<cosmic::app::Message<Message>>) {
        let cosmic_noise = CosmicNoise {
            core,
            popup: None,
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).ok(),
            track_list: files::load_data().unwrap_or_default(),
            currently_playing: HashMap::new(),
            state: PlaybackState::Stopped,
            error: None,
            // ..Default::default()batch
        };

        (cosmic_noise, Task::none())
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![widget::text::heading(fl!("app-title")).into()]
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::app::Message<Self::Message>> {
        if self.track_list.is_empty() {
            self.error = Some(Error::FileSystem)
        }

        match message {
            Message::Play(i) => match self.currently_playing.get_mut(&i) {
                Some(h) => match h.state() {
                    PlaybackState::Playing => {
                        h.pause(LINEAR_TWEEN);
                        self.track_list[i].state = PlaybackState::Paused;
                    }

                    PlaybackState::Paused => {
                        h.resume(Tween::default());
                        self.track_list[i].state = PlaybackState::Playing;
                    }
                    _ => {}
                },
                None => {
                    let settings = StreamingSoundSettings::new().loop_region(0.0..);
                    match StreamingSoundData::from_file(&self.track_list[i].path) {
                        Ok(sound_data) => {
                            match self.manager.as_mut() {
                                Some(am) => {
                                    match am.play(sound_data.with_settings(settings)) {
                                        Ok(h) => {
                                            self.currently_playing.insert(i, h);
                                        }
                                        Err(_e) => self.error = Some(Error::Handle),
                                    };
                                }
                                None => {
                                    self.error = Some(Error::PlayBack);
                                }
                            };
                        }
                        Err(e) => {
                            log::error!("Faild to play sound : {e}");
                            self.error = Some(Error::PlayBack);
                        }
                    };

                    self.track_list[i].state = PlaybackState::Playing;
                }
            },
            Message::VolumeChanged(level) => {
                println!("{:?}", level);
                let (f, s) = level;

                match self.currently_playing.get_mut(&s) {
                    Some(t) => {
                        t.set_volume(f, Tween::default());
                        self.track_list[s].volume_level = f;
                    }
                    None => {
                        log::info!("Could not change the volume!");
                    }
                }
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = cosmic::iced_core::window::Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::RESERVED, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE;
                    get_popup(popup_settings)
                };
            }
            // Message::PopupClosed(id) => {
            //     if self.popup.as_ref() == Some(&id) {
            //         self.popup = None;
            //     }
            // }
            Message::StopAll => {
                if !&self.currently_playing.is_empty() {
                    for (n, t) in &mut self.currently_playing {
                        t.stop(Tween::default());
                        self.track_list[*n].state = PlaybackState::Stopped;
                    }
                }
                self.currently_playing.clear();
                self.state = PlaybackState::Stopped;

                log::warn!("Could not stop  {:?}", self.currently_playing.is_empty());
            }
            Message::PauseAll => {
                if !&self.currently_playing.is_empty() {
                    self.currently_playing.iter_mut().for_each(|(n, t)| {
                        t.pause(Tween::default());
                        self.track_list[*n].state = PlaybackState::Paused;
                    });
                    self.state = PlaybackState::Paused;
                }
                log::warn!("Could not pause  {:?}", self.currently_playing.is_empty());
            }
            Message::ResumeAll => {
                if !&self.currently_playing.is_empty() {
                    self.currently_playing.iter_mut().for_each(|(n, t)| {
                        t.resume(Tween::default());
                        self.track_list[*n].state = PlaybackState::Playing;
                    });
                    self.state = PlaybackState::Playing;
                }
                log::warn!("Could not resume {:?}", self.currently_playing.is_empty());
            }
        }
        Task::none()
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
        let content = flex_row(get_elements(&self.track_list)).spacing(5);

        let play_pause = row![
            mouse_area(
                container(cosmic::widget::icon::from_name(
                    "io.github.bqwrongway.pause-symbolic",
                ))
                .width(20)
                .height(20),
            )
            .on_press(Message::PauseAll),
            mouse_area(
                container(cosmic::widget::icon::from_name(
                    "io.github.bqwrongway.play-symbolic",
                ))
                .width(20)
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
                    .height(20),
                )
                .on_press(Message::StopAll),
            )
            .push(horizontal_space())
            .push(text(fl!("app-title")))
            .push(horizontal_space())
            .push(play_pause)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(5)
            .align_y(Alignment::Center);
        let main_content = Column::new()
            .push(nav_row)
            .push(horizontal_rule(6))
            .push(horizontal_space().height(5))
            .push(scrollable(container(content).padding(Padding {
                // a workaround to keep slider away from content (its either this or fixed space with widget)
                top: 0.,
                right: 25.,
                left: 0.,
                bottom: 0.,
            })))
            .width(MAX_WIDTH * 4.)
            .height(MAX_HEIGHT * 5.)
            .padding(5);

        self.core
            .applet
            .popup_container(match self.error {
                Some(_) => Column::new().push(Space::new(Length::Fill,Length::Fill )).push(text("files not found on your system \n $HOME/.local/share/cosmic-noise/sounds \n is either empty or nonexistant").align_x(Center).size(Pixels::from(20))).push(Space::new(Length::Fill,Length::Fill)),
                None => main_content,
            }).auto_width(true).auto_height(true)
            .into()
    }
}

//need to deal with styling and global pause  resume
//maybe check some cursive fonts for the text
fn get_component(t: &NoiseTrack, i: usize) -> Column<Message> {
    cosmic::widget::column()
        .push(
            cosmic::widget::row()
                .push(
                    cosmic::iced::widget::text(uppercase_first(&t.name))
                        .class(match t.state {
                            PlaybackState::Paused => cosmic::style::Text::Default,
                            _ => cosmic::style::Text::Accent,
                        })
                        .size(14)
                        .shaping(Advanced)
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill),
                )
                .align_y(cosmic::iced_core::Alignment::Center),
        )
        .push(
            slider(-60.0..=40.0, t.volume_level, move |x| {
                Message::VolumeChanged((x, i))
            })
            .width(Length::Fill)
            .step(1.0)
            .height(10.0),
        )
        .spacing(SPACING)
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
                    .width(MAX_WIDTH)
                    .height(MAX_HEIGHT)
                    .class(match t.state {
                        PlaybackState::Paused => paused_contaner(),
                        PlaybackState::Playing => playing_contaner(),
                        _ => idle_container(),
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
    Handle,
}

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
