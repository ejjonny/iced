//! Change the appearance of a checkbox.
use iced_core::{Background, BorderRadius, Color};
use crate::animation::{self, Interpolable};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the checkbox.
    pub background: Background,
    /// The icon [`Color`] of the checkbox.
    pub icon_color: Color,
    /// The border radius of the checkbox.
    pub border_radius: BorderRadius,
    /// The border width of the checkbox.
    pub border_width: f32,
    /// The border [`Color`] of the checkbox.
    pub border_color: Color,
    /// The text [`Color`] of the checkbox.
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a checkbox.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a checkbox.
    fn active(&self, style: &Self::Style, is_checked: bool) -> Appearance;

    /// Produces the hovered [`Appearance`] of a checkbox.
    fn hovered(&self, style: &Self::Style, is_checked: bool) -> Appearance;
}

impl Interpolable for Appearance {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        Appearance {
            background: self.background.interpolated(other.background, ratio),
            icon_color: self.icon_color.interpolated(other.icon_color, ratio),
            border_radius: self.border_radius,
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
