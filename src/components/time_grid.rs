use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::styles::weekend_background;
use crate::ui_constants::{
    PADDING_SMALL, FONT_SIZE_SMALL, COLOR_DAY_CELL_BORDER,
    HOUR_ROW_HEIGHT, TIME_LABEL_WIDTH, BORDER_WIDTH_THIN
};

/// Information about a single day column in the time grid
#[derive(Clone)]
pub struct DayColumn {
    pub is_weekend: bool,
}

impl DayColumn {
    pub fn new(is_weekend: bool) -> Self {
        Self { is_weekend }
    }

    pub fn regular() -> Self {
        Self { is_weekend: false }
    }
}

/// Render a time grid with hourly slots
///
/// # Arguments
/// * `locale` - Locale preferences for time formatting
/// * `day_columns` - List of day columns to render (1 for day view, 7 for week view)
pub fn render_time_grid(
    locale: &LocalePreferences,
    day_columns: &[DayColumn],
) -> Element<'static, Message> {
    let mut grid = column().spacing(0);

    // Render 24 hours
    for hour in 0..24 {
        let mut hour_row = row().spacing(0);

        // Time label - use locale-aware formatting
        let time_label = locale.format_hour(hour);

        hour_row = hour_row.push(
            container(
                widget::text(time_label)
                    .size(FONT_SIZE_SMALL)
            )
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(HOUR_ROW_HEIGHT))
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Top)
            .style(|_theme: &cosmic::Theme| container::Style {
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            })
        );

        // Day columns
        for day_col in day_columns {
            let is_weekend = day_col.is_weekend;

            hour_row = hour_row.push(
                container(widget::text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(HOUR_ROW_HEIGHT))
                    .style(move |_theme: &cosmic::Theme| container::Style {
                        background: weekend_background(is_weekend),
                        border: Border {
                            width: BORDER_WIDTH_THIN,
                            color: COLOR_DAY_CELL_BORDER,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            );
        }

        grid = grid.push(hour_row);
    }

    grid.into()
}

/// Render the time column placeholder for all-day section headers
pub fn render_time_column_placeholder(height: f32) -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fixed(TIME_LABEL_WIDTH))
        .height(Length::Fixed(height))
        .into()
}

/// Style for a grid cell with border
pub fn grid_cell_style(is_weekend: bool) -> container::Style {
    container::Style {
        background: weekend_background(is_weekend),
        border: Border {
            width: BORDER_WIDTH_THIN,
            color: COLOR_DAY_CELL_BORDER,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for a bordered cell without weekend background
pub fn bordered_cell_style() -> container::Style {
    container::Style {
        border: Border {
            width: BORDER_WIDTH_THIN,
            color: COLOR_DAY_CELL_BORDER,
            ..Default::default()
        },
        ..Default::default()
    }
}
