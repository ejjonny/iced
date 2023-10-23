//! Show toggle controls using checkboxes.
use std::marker::PhantomData;
use std::{default, iter};

use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget::Tree;
use crate::core::{
    Alignment, Clipboard, Element, Layout, Length, Pixels, Rectangle, Shell,
    Widget,
};
use crate::{Row, Text};

use iced_renderer::core::{window, Background, BorderRadius};
use iced_runtime::program;
pub use iced_style::checkbox::{Appearance, StyleSheet};

use crate::core::overlay;

#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
pub struct Animating<'a, Message, T, Renderer = crate::Renderer>
where
    T: InterpolableState,
{
    child: Element<'a, Message, Renderer>,
    animation: Animated<T>,
    animation_update: Box<dyn Fn(Animated<T>) -> Message + 'a>,
}

#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
impl<'a, Message, T, Renderer> Animating<'a, Message, T, Renderer>
where
    T: InterpolableState,
{
    pub fn new<F>(
        child: Element<'a, Message, Renderer>,
        animation: Animated<T>,
        animation_update: F,
    ) -> Self
    where
        F: 'a + Fn(Animated<T>) -> Message,
    {
        Animating {
            child,
            animation,
            animation_update: Box::new(animation_update),
        }
    }
    pub fn duration(mut self, duration: std::time::Duration) -> Self {
        self.animation.duration = duration;
        self
    }

    pub fn timing(mut self, timing: Timing) -> Self {
        self.animation.timing = timing;
        self
    }
}

impl<'a, Message, T, Renderer> Widget<Message, Renderer>
    for Animating<'a, Message, T, Renderer>
where
    T: InterpolableState + Copy,
    Renderer: crate::core::Renderer,
{
    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_renderer::core::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.child
            .as_widget()
            .draw(state, renderer, theme, style, layout, cursor, viewport)
    }
    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.child
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                if self.animation.on_redraw_request_update(now) {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                    shell.publish((self.animation_update)(self.animation))
                }
            }
            _ => {}
        }
        iter::once(self)
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((animating, state), layout)| {
                animating.child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }
    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_renderer::core::widget::Operation<Message>,
    ) {
        self.child
            .as_widget()
            .operate(state, layout, renderer, operation)
    }
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.child.as_widget().layout(renderer, limits)
    }
    fn width(&self) -> Length {
        self.child.as_widget().width()
    }
    fn height(&self) -> Length {
        self.child.as_widget().height()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }
}

impl<'a, Message, T, Renderer> From<Animating<'a, Message, T, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    T: InterpolableState + Copy + 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(animating: Animating<'a, Message, T, Renderer>) -> Self {
        Self::new(animating)
    }
}

/// A box that can be checked.
///
/// # Example
///
/// ```no_run
/// # type Checkbox<'a, Message> =
/// #     iced_widget::Checkbox<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new("Toggle me!", is_checked, Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/checkbox.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Checkbox<'a, Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    state: Animated<CheckboxState>,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    on_hover: Box<dyn Fn(bool) -> Message + 'a>,
    label: String,
    width: Length,
    size: f32,
    spacing: f32,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    icon: Icon<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, Copy)]
pub struct CheckboxState {
    pub checked_amount: f32,
    pub hovered_amount: f32,
}
#[allow(missing_docs)]
impl CheckboxState {
    pub fn check(&mut self, value: bool) {
        self.checked_amount = if value { 1.0 } else { 0.0 }
    }
}
impl InterpolableState for CheckboxState {
    fn interpolate(self, other: Self, ratio: f32) -> Self {
        Self {
            checked_amount: self
                .checked_amount
                .interpolated(other.checked_amount, ratio),
            hovered_amount: self
                .hovered_amount
                .interpolated(other.hovered_amount, ratio),
        }
    }
}
#[allow(missing_docs)]
impl CheckboxState {
    pub fn new(is_checked: bool, is_hovered: bool) -> Self {
        Self {
            checked_amount: if is_checked { 1.0 } else { 0.0 },
            hovered_amount: if is_hovered { 1.0 } else { 0.0 },
        }
    }
}

#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
#[derive(Default, Debug)]
pub struct Animated<T>
where
    T: InterpolableState,
{
    linear_progress: f32,
    pub started: Option<std::time::Instant>,
    pub last: Option<std::time::Instant>,
    pub from: T,
    pub to: Option<T>,
    pub duration: std::time::Duration,
    pub timing: Timing,
}
#[allow(missing_docs)]
pub trait InterpolableState {
    fn interpolate(self, other: Self, ratio: f32) -> Self;
}
impl<T> Clone for Animated<T>
where
    T: InterpolableState + Clone,
{
    fn clone(&self) -> Self {
        Self {
            linear_progress: self.linear_progress.clone(),
            started: self.started.clone(),
            last: self.last.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            duration: self.duration.clone(),
            timing: self.timing.clone(),
        }
    }
}
impl<T> Copy for Animated<T> where T: InterpolableState + Copy {}
// impl Animated<bool> {
//     fn amount(self) -> f32 {
//         0.0
// if self.animating() {
//     return if self.to.unwrap_or(self.from) {
//         self.timed_progress()
//     } else {
//         1.0 - self.timed_progress()
//     };
// } else {
//     return if self.from { 1.0 } else { 0.0 };
// }
//     }
// }

#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
impl<T> Animated<T>
where
    T: InterpolableState + Copy,
{
    pub fn new(value: T) -> Self {
        Animated {
            linear_progress: 0.0,
            started: None,
            last: None,
            from: value,
            to: None,
            duration: std::time::Duration::from_millis(500),
            timing: Timing::Linear,
        }
    }
    pub fn interpolated_value(self) -> T {
        if let Some(other) = self.to {
            self.from.interpolate(other, self.timed_progress())
        } else {
            self.from
        }
    }
    pub fn real_value(&self) -> T {
        self.to.unwrap_or(self.from)
    }
    pub fn transition<F>(&mut self, update: F)
    where
        F: Fn(&mut T),
    {
        let mut target = self.from.clone();
        update(&mut target);
        if self.animating() {
            // Snapshot current state as the new animation origin
            self.from = self.from.interpolate(self.to.unwrap_or(self.from), self.linear_progress);
            // How long should this take?
            // This progress represents multiple animated axes
            // Ideally we just go the same speed but how do we conceptualize distance?
            // Every animatable piece of data should probably be it's own 1D axis with it's
            // own animated state
            // The current approach bundles everything...
            self.linear_progress = 1.0 - self.linear_progress;
            self.to = Some(target);
        }
        self.last = None;
        self.started = Some(std::time::Instant::now());
        self.to = Some(target);
    }
    pub fn on_redraw_request_update(
        &mut self,
        now: std::time::Instant,
    ) -> bool {
        if let Some(start) = self.started {
            let elapsed = (now - self.last.unwrap_or(start)).as_millis() as f32;
            let duration = self.duration.as_millis() as f32;
            self.linear_progress += elapsed / duration;
            if self.linear_progress >= 1.0 || self.linear_progress.is_nan() {
                if let Some(to) = self.to {
                    self.from = to;
                }
                self.linear_progress = 0.0;
                self.started = None;
                self.to = None;
                self.last = None;
            }
            self.last = Some(now);
            return true;
        }
        false
    }
    pub fn timed_progress(self) -> f32 {
        self.timing.timing(self.linear_progress)
    }

    pub fn animating(self) -> bool {
        self.to.is_some()
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub enum Timing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom,
}

impl Timing {
    fn timing(self, linear_progress: f32) -> f32 {
        let x = linear_progress;
        let pi = std::f32::consts::PI;
        match self {
            Timing::Linear => linear_progress,
            Timing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Timing::EaseOut => f32::sin((x * pi) / 2.0),
            Timing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            _ => linear_progress,
        }
    }
}

impl<'a, Message, Renderer> Checkbox<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    /// The default size of a [`Checkbox`].
    const DEFAULT_SIZE: f32 = 20.0;

    /// The default spacing of a [`Checkbox`].
    const DEFAULT_SPACING: f32 = 15.0;

    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    pub fn new<F, G>(
        label: impl Into<String>,
        state: Animated<CheckboxState>,
        on_toggle: F,
        on_hover: G,
    ) -> Self
    where
        F: 'a + Fn(bool) -> Message,
        G: 'a + Fn(bool) -> Message,
    {
        Checkbox {
            state,
            on_toggle: Box::new(on_toggle),
            on_hover: Box::new(on_hover),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            icon: Icon {
                font: Renderer::ICON_FONT,
                code_point: Renderer::CHECKMARK_ICON,
                size: None,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Basic,
            },
            style: Default::default(),
        }
    }

    /// Sets the size of the [`Checkbox`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Checkbox`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the spacing between the [`Checkbox`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the text size of the [`Checkbox`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Checkbox`].
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Checkbox`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the [`Font`] of the text of the [`Checkbox`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Icon`] of the [`Checkbox`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = icon;
        self
    }

    /// Sets the style of the [`Checkbox`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Checkbox<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center)
            .push(Row::new().width(self.size).height(self.size))
            .push(
                Text::new(&self.label)
                    .font(self.font.unwrap_or_else(|| renderer.default_font()))
                    .width(self.width)
                    .size(
                        self.text_size
                            .unwrap_or_else(|| renderer.default_size()),
                    )
                    .line_height(self.text_line_height)
                    .shaping(self.text_shaping),
            )
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let mouse_over = cursor.is_over(layout.bounds());

                if mouse_over {
                    shell.publish((self.on_toggle)(
                        !(self.state.to.unwrap_or(self.state.from).checked_amount == 1.0),
                    ));
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // if cursor.is_over(layout.bounds()) {
                //     dbg!(position);
                // }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let amount = self.state.interpolated_value().checked_amount;
        dbg!(amount);

        let is_mouse_over = cursor.is_over(layout.bounds());

        let mut children = layout.children();

        let custom_style_on = if is_mouse_over {
            theme.hovered(&self.style, true)
        } else {
            theme.active(&self.style, true)
        };
        let custom_style_off = if is_mouse_over {
            theme.hovered(&self.style, false)
        } else {
            theme.active(&self.style, false)
        };
        let interpolated_style =
            custom_style_off.interpolated(custom_style_on, amount);

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: interpolated_style.border_radius,
                    border_width: interpolated_style.border_width,
                    border_color: interpolated_style.border_color,
                },
                interpolated_style.background,
            );

            let Icon {
                font,
                code_point,
                size,
                line_height,
                shaping,
            } = &self.icon;
            let size = size.unwrap_or(bounds.height * 0.7);

            let mut transparent_icon = interpolated_style.icon_color;
            transparent_icon.a = 0.0;
            if amount != 0.0 {
                renderer.fill_text(text::Text {
                    content: &code_point.to_string(),
                    font: *font,
                    size,
                    line_height: *line_height,
                    bounds: Rectangle {
                        x: bounds.center_x(),
                        y: bounds.center_y(),
                        ..bounds
                    },
                    color: interpolated_style.icon_color,
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: *shaping,
                });
            }
        }

        {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                &self.label,
                self.text_size,
                self.text_line_height,
                self.font,
                crate::text::Appearance {
                    color: interpolated_style.text_color,
                },
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
                self.text_shaping,
            );
        }
    }
}

impl<'a, Message, Renderer> From<Checkbox<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn from(
        checkbox: Checkbox<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}

/// The icon in a [`Checkbox`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    /// Font that will be used to display the `code_point`,
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// Font size of the content.
    pub size: Option<f32>,
    /// The line height of the icon.
    pub line_height: text::LineHeight,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}

trait Interpolable {
    fn interpolated(self, other: Self, ratio: f32) -> Self;
}
impl Interpolable for crate::core::Color {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self.mixed(other, ratio)
    }
}
impl Interpolable for f32 {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self * (1.0 - ratio) + other * ratio
    }
}
impl Interpolable for Background {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Background::Color(a), Background::Color(b)) => {
                return Background::Color(a.interpolated(b, ratio))
            }
            _ => return other,
        }
    }
}
impl<T> Interpolable for Option<T>
where
    T: Interpolable + Copy,
{
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.interpolated(b, ratio)),
            _ => other,
        }
    }
}
impl Interpolable for BorderRadius {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self
        // self.0.iter().zip(other.0.iter()).map |a, b| { a.interpolated(b, ratio) }
    }
}

impl Interpolable for Appearance {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        Appearance {
            background: self.background.interpolated(other.background, ratio),
            icon_color: self.icon_color.interpolated(other.icon_color, ratio),
            border_radius: self
                .border_radius
                .interpolated(other.border_radius, ratio),
            border_width: self
                .border_width
                .interpolated(other.border_width, ratio),
            border_color: self
                .border_color
                .interpolated(other.border_color, ratio),
            text_color: self.text_color.interpolated(other.text_color, ratio),
        }
    }
}
