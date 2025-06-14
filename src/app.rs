use crate::fl;
use cosmic::app::Core;
use cosmic::iced::Alignment::Center;
use cosmic::iced::Length::{Fill, Shrink};
use cosmic::iced::Limits;
use cosmic::iced_core::window::Id;
use cosmic::iced_widget::text::Shaping::Advanced;
use cosmic::iced_widget::{horizontal_rule, row, scrollable, text};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::theme::iced::Slider;
use cosmic::widget::text::heading;
use cosmic::widget::{
    container, flex_row, horizontal_space, mouse_area, slider, Column, Row, Space,
};
use cosmic::{style, Application, Element, Task};
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
use crate::utils::files::{self, NoiseTrack};
use crate::utils::ui_helpers::{idle_container, paused_contaner, playing_contaner};

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
    Loaded(Result<Vec<NoiseTrack>, Error>),
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
    ) -> (CosmicNoise, cosmic::Task<cosmic::Action<Message>>) {
        let cosmic_noise = CosmicNoise {
            core,
            popup: None,
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).ok(),
            track_list: vec![],
            currently_playing: HashMap::new(),
            state: PlaybackState::Stopped,
            error: None,
            // ..Default::default()batch
        };

        (
            cosmic_noise,
            Task::perform(files::load_data(), |f| {
                cosmic::Action::App(Message::Loaded(f))
                // Message::Loaded(f)
            }),
        )
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![heading(fl!("app-title")).into()]
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Loaded(v) => match v {
                Ok(tracks) => self.track_list = tracks,
                Err(e) => self.error = Some(e),
            },
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
                    match play_sound(i, &self.track_list, &mut self.manager, settings) {
                        Ok(result) => {
                            let (index, handle) = result;
                            self.currently_playing.insert(index, handle);
                        }
                        Err(_e) => self.error = Some(Error::PlayBack),
                    }

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
        //how to load icon, from system icons, but still fallback to custom one in case of error
        self.core
            .applet
            // .icon_button_from_handle(Handle {
            //     symbolic: true,
            //     data: widget::icon::Data::Svg(svg::Handle::from_path("//pathto")),
            // })
            .icon_button("io.github.bqwrongway.wave-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        //need to pay attention to flex row, since its inside of scrollable it might need to be wrapped by the container (no width/noheight settigns)
        let content = flex_row(get_elements(&self.track_list)).spacing(5);

        let play_pause = row![
            mouse_area(container(
                cosmic::widget::icon::from_name("io.github.bqwrongway.pause-symbolic",).size(20)
            ))
            .on_press(Message::PauseAll),
            mouse_area(container(
                cosmic::widget::icon::from_name("io.github.bqwrongway.play-symbolic",).size(20)
            ))
            .on_press(Message::ResumeAll)
        ]
        .push(Space::new(10, 5))
        .spacing(10);
        let nav_row = Row::new()
            .push(
                mouse_area(container(
                    cosmic::widget::icon::from_name("io.github.bqwrongway.stop-symbolic").size(20),
                ))
                .on_press(Message::StopAll),
            )
            .push(horizontal_space())
            .push(text(fl!("app-title")))
            .push(horizontal_space())
            .push(play_pause)
            .width(Fill)
            .height(Shrink)
            .padding(5)
            .align_y(Center);
        let main_content = Column::new()
            .push(nav_row)
            .push_maybe(self.error.is_some().then(|| {
                text(fl!("not-found"))
                    .class(style::Text::Custom(|t| text::Style {
                        color: Some(t.cosmic().destructive_text_color().into()),
                    }))
                    .size(14.)
                    .width(Fill)
                    .align_x(Center)
                    .wrapping(text::Wrapping::Word)
            }))
            .push(horizontal_rule(6))
            .push(horizontal_space().height(5))
            .push(scrollable(row![content].push(Space::new(18, 1))))
            .width(MAX_WIDTH * 4.)
            .height(MAX_HEIGHT * 5.)
            .padding(10);

        self.core
            .applet
            .popup_container(main_content)
            .auto_width(true)
            .auto_height(true)
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
                            PlaybackState::Paused => style::Text::Default,
                            _ => style::Text::Accent,
                        })
                        .size(14)
                        .shaping(Advanced)
                        .height(Fill)
                        .align_y(Center)
                        .align_x(Center)
                        .width(Fill),
                )
                .align_y(Center),
        )
        .push(
            slider(-60.0..=40.0, t.volume_level, move |x| {
                Message::VolumeChanged((x, i))
            })
            .class(Slider::Standard)
            .width(Fill)
            .step(1.0)
            .height(10.0),
        )
        .spacing(SPACING)
        .width(Fill)
        .height(Fill)
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
#[derive(Debug, Clone)]
pub enum Error {
    FileSystem,
    PlayBack,
    Handle,
}

pub fn play_sound(
    i: usize,
    tracks: &[NoiseTrack],
    manager: &mut Option<AudioManager>,
    settings: StreamingSoundSettings,
) -> Result<(usize, StreamingSoundHandle<FromFileError>), Error> {
    match StreamingSoundData::from_file(&tracks[i].path) {
        Ok(sound_data) => match manager.as_mut() {
            Some(am) => match am.play(sound_data.with_settings(settings)) {
                Ok(h) => Ok((i, h)),
                Err(_e) => Err(Error::Handle),
            },
            None => Err(Error::PlayBack),
        },
        Err(e) => {
            log::error!("Faild to play sound : {e}");
            Err(Error::PlayBack)
        }
    }
}
