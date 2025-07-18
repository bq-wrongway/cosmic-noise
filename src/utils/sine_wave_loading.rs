use iced::Length::Fill;
use iced::advanced::layout;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::border::rounded;
use iced::mouse;
use iced::time::Instant;
use iced::window;
use iced::{Background, Color, Element, Event, Length, Rectangle, Size};

use std::f32::consts::PI;
use std::time::Duration;

#[allow(missing_debug_implementations)]
pub struct SineWaveLoading<'a, Theme>
where
    Theme: StyleSheet<Style = Style>,
{
    width: Length,
    height: Length,
    style_fn: Option<Box<dyn Fn(&Theme) -> Style + 'a>>,
    bar_count: usize,
    radius: f32,
    running: bool,
    cycle_duration: Duration,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, Theme> SineWaveLoading<'a, Theme>
where
    Theme: StyleSheet<Style = Style>,
{
    pub fn new() -> Self {
        Self {
            width: Fill,
            height: Fill,
            style_fn: None,
            bar_count: 7,
            running: true,
            cycle_duration: Duration::from_millis(900),
            radius: 0.0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn style<F>(mut self, style_fn: F) -> Self
    where
        F: Fn(&Theme) -> Style + 'a,
    {
        self.style_fn = Some(Box::new(style_fn));
        self
    }

    pub fn bar_count(mut self, count: usize) -> Self {
        self.bar_count = count;
        self
    }

    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    // Set whether the animation is running
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }
}

impl<Theme> Default for SineWaveLoading<'_, Theme>
where
    Theme: StyleSheet<Style = Style>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
struct State {
    last_update: Instant,
    phase: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            phase: 0.0,
        }
    }
}

impl State {
    fn update(&mut self, now: Instant, cycle_duration: Duration) {
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;
        let cycle_sec = cycle_duration.as_secs_f32();
        self.phase = (self.phase + 2.0 * PI * (elapsed.as_secs_f32() / cycle_sec)) % (2.0 * PI);
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for SineWaveLoading<'a, Theme>
where
    Message: Clone + 'a,
    Theme: StyleSheet<Style = Style> + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if self.running {
                state.update(*now, self.cycle_duration);
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = if let Some(ref style_fn) = self.style_fn {
            style_fn(theme)
        } else {
            Style::default()
        };
        let custom_style = theme.appearance(&style);
        let state = tree.state.downcast_ref::<State>();

        // Background fill
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            Background::Color(custom_style.background_color),
        );

        // Bars animate as sine wave, centered vertically
        let bar_count = self.bar_count;
        let width = bounds.width;
        let height = bounds.height;
        let w = width * 0.8;
        let h = height * 0.6;
        let x_start = bounds.x + (width - w) / 2.0;
        let bar_width = w / (bar_count as f32 * 2.0);
        let bar_spacing = w / (bar_count as f32);

        // Phase offset for a wave pattern
        let wave_length = bar_count as f32;
        for i in 0..bar_count {
            let x = x_start + i as f32 * bar_spacing;
            // Each bar offset by phase, so all bars form a wave
            let offset = i as f32 * (2.0 * PI / wave_length);
            let wave = ((state.phase + offset).sin() + 1.0) / 2.0; // 0..1
            let bar_h = h * (0.2 + 0.8 * wave); // minimum 20% height, up to full

            // Center the bar vertically in the bounds:
            let y = bounds.y + (bounds.height - bar_h) / 2.0;

            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x,
                        y,
                        width: bar_width,
                        height: bar_h,
                    },
                    border: rounded(self.radius),
                    ..Quad::default()
                },
                Background::Color(custom_style.bar_color),
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<SineWaveLoading<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: StyleSheet<Style = Style> + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(wave: SineWaveLoading<'a, Theme>) -> Self {
        Self::new(wave)
    }
}

// --- Style infrastructure is exactly the same as before ---

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub color: Color,
    pub background_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    // The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
    pub background_color: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            bar_color: Color::BLACK,
            background_color: Color::TRANSPARENT,
        }
    }
}

// A set of rules that dictate the style of an indicator.
pub trait StyleSheet {
    // The supported style of the [`StyleSheet`].
    type Style: Default;
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

impl StyleSheet for iced::Theme {
    type Style = Style;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        Appearance {
            bar_color: style.color,
            background_color: style.background_color,
        }
    }
}
