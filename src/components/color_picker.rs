use cosmic::iced::{Color, Length};
use cosmic::widget::{button, column, container, row};
use cosmic::{widget, Element};

use crate::message::Message;
use crate::styles::color_button_style;
use crate::ui_constants::{
    COLOR_BUTTON_SIZE_SMALL, COLOR_BUTTON_SIZE_MEDIUM, COLOR_BUTTON_SIZE_LARGE,
    SPACING_COLOR_GRID, SPACING_COLOR_CONTAINER, PADDING_STANDARD,
    COLOR_DEFAULT_GRAY, COLOR_BORDER_LIGHT, COLOR_BORDER_SELECTED,
    BORDER_WIDTH_HIGHLIGHT, BORDER_WIDTH_SELECTED
};

/// Predefined color palette for calendars (full list with names)
pub const CALENDAR_COLORS: &[(&str, &str)] = &[
    // Blues
    ("#3B82F6", "Blue"),
    ("#0EA5E9", "Sky Blue"),
    ("#2563EB", "Royal Blue"),
    ("#1E40AF", "Dark Blue"),
    // Purples
    ("#8B5CF6", "Purple"),
    ("#A78BFA", "Light Purple"),
    ("#7C3AED", "Violet"),
    ("#6D28D9", "Deep Purple"),
    // Greens
    ("#10B981", "Green"),
    ("#34D399", "Emerald"),
    ("#059669", "Dark Green"),
    ("#047857", "Forest Green"),
    // Yellows/Oranges
    ("#F59E0B", "Amber"),
    ("#FBBF24", "Yellow"),
    ("#F97316", "Orange"),
    ("#EA580C", "Dark Orange"),
    // Reds/Pinks
    ("#EF4444", "Red"),
    ("#F87171", "Light Red"),
    ("#DC2626", "Crimson"),
    ("#EC4899", "Pink"),
    // Others
    ("#6366F1", "Indigo"),
    ("#14B8A6", "Teal"),
    ("#06B6D4", "Cyan"),
    ("#6B7280", "Gray"),
];

/// Quick picker color grid - 20 colors in 4 rows of 5
/// Used by both the sidebar quick picker and the calendar dialog
pub const QUICK_PICKER_COLORS: [[&str; 5]; 4] = [
    // Row 1: Blues
    ["#3B82F6", "#0EA5E9", "#2563EB", "#1E40AF", "#06B6D4"],
    // Row 2: Purples and Pinks
    ["#8B5CF6", "#A78BFA", "#7C3AED", "#EC4899", "#DB2777"],
    // Row 3: Greens and Teals
    ["#10B981", "#34D399", "#059669", "#14B8A6", "#0D9488"],
    // Row 4: Oranges, Yellows, Reds
    ["#F59E0B", "#FBBF24", "#F97316", "#EF4444", "#DC2626"],
];

/// Render a color indicator button
pub fn render_color_indicator<'a>(
    calendar_id: String,
    current_color: &str,
    size: f32,
) -> Element<'a, Message> {
    let color = parse_hex_color(current_color).unwrap_or(COLOR_DEFAULT_GRAY);

    button::custom(
        container(widget::text(""))
            .width(size)
            .height(size)
            .style(move |_theme: &cosmic::Theme| {
                color_button_style(color, size, BORDER_WIDTH_HIGHLIGHT, COLOR_BORDER_LIGHT)
            })
    )
    .on_press(Message::ToggleColorPicker(calendar_id))
    .padding(0)
    .into()
}

/// Render a color picker palette with all available colors
#[allow(dead_code)]
pub fn render_color_palette<'a>(calendar_id: String) -> Element<'a, Message> {
    let mut color_grid = column().spacing(SPACING_COLOR_CONTAINER);

    // Split colors into rows (6 per row)
    for row_colors in CALENDAR_COLORS.chunks(6) {
        let mut color_row = row().spacing(SPACING_COLOR_CONTAINER);

        for (hex, _name) in row_colors {
            let color = parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_GRAY);
            let hex_owned = hex.to_string();
            let calendar_id_clone = calendar_id.clone();

            let color_button = button::custom(
                container(widget::text(""))
                    .width(COLOR_BUTTON_SIZE_MEDIUM)
                    .height(COLOR_BUTTON_SIZE_MEDIUM)
                    .style(move |_theme: &cosmic::Theme| {
                        color_button_style(color, COLOR_BUTTON_SIZE_MEDIUM, BORDER_WIDTH_HIGHLIGHT, COLOR_BORDER_LIGHT)
                    })
            )
            .on_press(Message::ChangeCalendarColor(calendar_id_clone, hex_owned))
            .padding(0);

            color_row = color_row.push(
                container(color_button)
                    .width(Length::Fixed(COLOR_BUTTON_SIZE_LARGE))
                    .height(Length::Fixed(COLOR_BUTTON_SIZE_LARGE))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            );
        }

        color_grid = color_grid.push(color_row);
    }

    container(color_grid)
        .padding(PADDING_STANDARD)
        .into()
}

/// Render a compact color picker with 20 colors in a 4x5 grid
pub fn render_quick_color_picker<'a>(
    calendar_id: String,
    current_color: &str,
) -> Element<'a, Message> {
    let mut color_grid = column().spacing(SPACING_COLOR_GRID);

    for row_colors in QUICK_PICKER_COLORS {
        let mut color_row = row().spacing(SPACING_COLOR_GRID);

        for hex in row_colors {
            let color = parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_GRAY);
            let hex_owned = hex.to_string();
            let calendar_id_clone = calendar_id.clone();
            let is_selected = current_color == hex;

            let border_width = if is_selected { BORDER_WIDTH_SELECTED } else { BORDER_WIDTH_HIGHLIGHT };
            let border_color = if is_selected { COLOR_BORDER_SELECTED } else { COLOR_BORDER_LIGHT };

            let color_button = button::custom(
                container(widget::text(""))
                    .width(COLOR_BUTTON_SIZE_SMALL)
                    .height(COLOR_BUTTON_SIZE_SMALL)
                    .style(move |_theme: &cosmic::Theme| {
                        color_button_style(color, COLOR_BUTTON_SIZE_SMALL, border_width, border_color)
                    })
            )
            .on_press(Message::ChangeCalendarColor(calendar_id_clone, hex_owned))
            .padding(0);

            color_row = color_row.push(color_button);
        }

        color_grid = color_grid.push(color_row);
    }

    color_grid.into()
}

/// Parse hex color string (e.g., "#RRGGBB") to iced Color
pub fn parse_hex_color(hex: &str) -> Result<Color, ()> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;

    Ok(Color::from_rgb8(r, g, b))
}

/// Parse hex color string safely, returning a default gray color on failure.
/// Use this instead of `parse_hex_color().unwrap_or()` pattern.
pub fn parse_color_safe(hex: &str) -> Color {
    parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_GRAY)
}
