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
use iced_style::animation::{Interpolable, AnimatedValue, Animatable};
pub use iced_style::checkbox::{Appearance, StyleSheet};

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
    state: CheckboxState,
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

#[derive(Debug, Clone, Copy)]
pub struct CheckboxState {
    pub checked_amount: AnimatedValue<std::time::Instant>,
    pub hovered_amount: AnimatedValue<std::time::Instant>,
}

impl CheckboxState {
    pub fn check(&mut self, value: bool) {
        self.checked_amount.transition(std::time::Instant::now(), |current| {
           *current = if value { 1.0 } else { 0.0 }
        });
    }
    pub fn hover(&mut self, value: bool) {
        self.hovered_amount.transition(std::time::Instant::now(), |current| {
           *current = if value { 1.0 } else { 0.0 }
        });
    }
}

impl CheckboxState {
    pub fn new(is_checked: bool, is_hovered: bool) -> Self {
        Self {
            checked_amount: AnimatedValue::new(if is_checked { 1.0 } else { 0.0 }),
            hovered_amount: AnimatedValue::new(if is_hovered { 1.0 } else { 0.0 }),
        }
    }
}

impl Animatable for CheckboxState {
    fn on_redraw_request_update(
        &mut self,
        now: std::time::Instant,
    ) -> bool {
        self.checked_amount.tick(now) || self.hovered_amount.tick(now)
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
        state: CheckboxState,
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
                        !(self.state.checked_amount.real_value() == 1.0),
                    ));
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let mouse_over = cursor.is_over(layout.bounds());
                let currently_hovered = self.state.hovered_amount.real_value() == 1.0;
                if mouse_over && !currently_hovered {
                    shell.publish((self.on_hover)(true));
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                } else if !mouse_over && currently_hovered {
                    shell.publish((self.on_hover)(false));
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
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
        let checked_amount = self.state.checked_amount.timed_progress();
        let hovered_amount = self.state.hovered_amount.timed_progress();
        dbg!(checked_amount);

        let mut children = layout.children();

        let checked_interpolated =
            theme.active(&self.style, false).interpolated(theme.active(&self.style, true), checked_amount);
        let hovered_interpolated =
            theme.hovered(&self.style, false).interpolated(theme.hovered(&self.style, true), checked_amount);
        let interpolated_style = checked_interpolated.interpolated(hovered_interpolated, hovered_amount);

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

            if checked_amount != 0.0 {
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
