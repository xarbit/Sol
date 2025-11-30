use cosmic::iced::{Background, Border, Color, Shadow, Vector};
use cosmic::widget::container;
use crate::ui_constants::{
    SHADOW_OPACITY, SHADOW_OFFSET_X, SHADOW_OFFSET_Y, SHADOW_BLUR_RADIUS,
    BORDER_RADIUS, BORDER_WIDTH_HIGHLIGHT, BORDER_WIDTH_NORMAL,
    COLOR_DAY_CELL_BORDER, COLOR_WEEKEND_BACKGROUND, COLOR_TODAY_BLUE
};

/// Style for the overlay sidebar in mobile/condensed mode
pub fn overlay_sidebar_style(theme: &cosmic::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.cosmic().background.base.into(),
        )),
        border: Border {
            width: 0.0,
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, SHADOW_OPACITY),
            offset: Vector::new(SHADOW_OFFSET_X, SHADOW_OFFSET_Y),
            blur_radius: SHADOW_BLUR_RADIUS,
        },
        ..Default::default()
    }
}

/// Style for today indicator - filled background with accent color
/// Used consistently across day headers in week/day views
pub fn today_filled_style(theme: &cosmic::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.cosmic().accent_color().into()
        )),
        border: Border {
            radius: BORDER_RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for selected day cell - border with accent color, preserving weekend background
pub fn selected_day_style(theme: &cosmic::Theme, is_weekend: bool) -> container::Style {
    container::Style {
        background: if is_weekend {
            Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
        } else {
            None
        },
        border: Border {
            color: theme.cosmic().accent_color().into(),
            width: BORDER_WIDTH_HIGHLIGHT,
            radius: BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

/// Style for today's day number circle - fixed blue background with white text
/// Consistent across all themes for easy recognition
pub fn today_circle_style(_theme: &cosmic::Theme, size: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_TODAY_BLUE)),
        border: Border {
            radius: (size / 2.0).into(), // Circular
            ..Default::default()
        },
        // White text for contrast on blue background
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

/// Style for regular day cell with optional weekend background
pub fn day_cell_style(is_weekend: bool) -> container::Style {
    container::Style {
        background: if is_weekend {
            Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
        } else {
            None
        },
        border: Border {
            color: COLOR_DAY_CELL_BORDER,
            width: BORDER_WIDTH_NORMAL,
            radius: BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

/// Style for adjacent month day cells (previous/next month) - grayed out
pub fn adjacent_month_day_style() -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: COLOR_DAY_CELL_BORDER,
            width: BORDER_WIDTH_NORMAL,
            radius: BORDER_RADIUS.into(),
        },
        // Gray text for adjacent month days
        text_color: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
        ..Default::default()
    }
}

/// Style for selected adjacent month day cells - grayed out text with accent border
pub fn adjacent_month_selected_style(theme: &cosmic::Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: theme.cosmic().accent_color().into(),
            width: BORDER_WIDTH_HIGHLIGHT,
            radius: BORDER_RADIUS.into(),
        },
        // Gray text for adjacent month days (same as non-selected)
        text_color: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
        ..Default::default()
    }
}

/// Style for a circular color button
pub fn color_button_style(color: Color, size: f32, border_width: f32, border_color: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: (size / 2.0).into(),
            width: border_width,
            color: border_color,
        },
        ..Default::default()
    }
}

/// Style for day cells in a drag selection range
/// Uses a semi-transparent accent color background
pub fn selection_highlight_style(theme: &cosmic::Theme, _is_weekend: bool) -> container::Style {
    let accent = theme.cosmic().accent_color();
    // Light semi-transparent accent background for selection
    let selection_bg = Color::from_rgba(accent.red, accent.green, accent.blue, 0.2);

    container::Style {
        background: Some(Background::Color(selection_bg)),
        border: Border {
            color: Color::from_rgba(accent.red, accent.green, accent.blue, 0.4),
            width: BORDER_WIDTH_NORMAL,
            radius: BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

/// Style for adjacent month day cells in selection range
pub fn adjacent_month_selection_style(theme: &cosmic::Theme) -> container::Style {
    let accent = theme.cosmic().accent_color();
    // Lighter selection for adjacent months
    let selection_bg = Color::from_rgba(accent.red, accent.green, accent.blue, 0.1);

    container::Style {
        background: Some(Background::Color(selection_bg)),
        border: Border {
            color: Color::from_rgba(accent.red, accent.green, accent.blue, 0.3),
            width: BORDER_WIDTH_NORMAL,
            radius: BORDER_RADIUS.into(),
        },
        // Gray text for adjacent month days
        text_color: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
        ..Default::default()
    }
}

/// Style for a day cell that is the current drop target during event drag
/// Uses a dashed-like appearance with accent color to indicate where the event will land
pub fn drag_target_style(theme: &cosmic::Theme, is_weekend: bool) -> container::Style {
    let accent = theme.cosmic().accent_color();
    // Highlight background to show this is the drop target
    let target_bg = Color::from_rgba(accent.red, accent.green, accent.blue, 0.25);

    container::Style {
        background: Some(Background::Color(if is_weekend {
            // Blend with weekend background
            Color::from_rgba(
                (COLOR_WEEKEND_BACKGROUND.r + target_bg.r) / 2.0,
                (COLOR_WEEKEND_BACKGROUND.g + target_bg.g) / 2.0,
                (COLOR_WEEKEND_BACKGROUND.b + target_bg.b) / 2.0,
                0.5,
            )
        } else {
            target_bg
        })),
        border: Border {
            color: Color::from_rgba(accent.red, accent.green, accent.blue, 0.8),
            width: BORDER_WIDTH_HIGHLIGHT,
            radius: BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}
