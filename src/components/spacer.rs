//! Spacer helper functions for creating empty space in layouts.
//!
//! These functions replace the repeated pattern of `container(widget::text(""))`.

use cosmic::iced::Length;
use cosmic::widget::container;
use cosmic::{widget, Element};

use crate::message::Message;

/// Creates a spacer with the specified width and height.
pub fn spacer(width: Length, height: Length) -> Element<'static, Message> {
    container(widget::text(""))
        .width(width)
        .height(height)
        .into()
}

/// Creates a spacer with fixed pixel dimensions.
pub fn fixed_spacer(width: f32, height: f32) -> Element<'static, Message> {
    spacer(Length::Fixed(width), Length::Fixed(height))
}

/// Creates a vertical spacer that fills available width.
pub fn vertical_spacer(height: f32) -> Element<'static, Message> {
    spacer(Length::Fill, Length::Fixed(height))
}

/// Creates a horizontal spacer that fills available height.
pub fn horizontal_spacer(width: f32) -> Element<'static, Message> {
    spacer(Length::Fixed(width), Length::Fill)
}

/// Creates a spacer that fills all available space.
pub fn fill_spacer() -> Element<'static, Message> {
    spacer(Length::Fill, Length::Fill)
}

/// Creates a spacer with shrink width and fixed height.
pub fn shrink_vertical_spacer(height: f32) -> Element<'static, Message> {
    spacer(Length::Shrink, Length::Fixed(height))
}
