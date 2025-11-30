mod calendar_dialog;
mod calendar_dialog_callbacks;
mod calendar_list;
pub mod color_picker;
mod day_cell;
mod day_header;
mod display_mode;
mod event_chip;
mod event_dialog;
mod event_dialog_callbacks;
mod header_menu;
mod mini_calendar;
mod time_grid;
mod time_picker;
mod toolbar;

pub use calendar_dialog::{render_calendar_dialog, render_delete_calendar_dialog};
pub use calendar_list::render_calendar_list;
pub use event_dialog::render_event_dialog;
pub use color_picker::{render_color_indicator, render_quick_color_picker, parse_hex_color};
pub use day_cell::{render_day_cell_with_events, DayCellConfig};
pub use day_header::{render_day_header, DayHeaderConfig};
pub use event_chip::{render_quick_event_input, render_spanning_quick_event_input, render_compact_events, render_unified_events_with_selection, quick_event_input_id, DisplayEvent};
pub use header_menu::{render_header_end, render_header_start};
pub use mini_calendar::render_mini_calendar;
pub use time_grid::{render_time_grid, render_time_column_placeholder, grid_cell_style, bordered_cell_style, DayColumn};
// time_picker is used internally by event_dialog
#[allow(unused_imports)]
pub use time_picker::render_time_picker;
pub use toolbar::render_toolbar;
pub use display_mode::{EventDisplayMode, calculate_display_mode, should_use_compact};

// These callback structs are available for future use when we complete the refactoring
// to make dialogs generic over message type (like time_picker.rs)
#[allow(unused_imports)]
pub use calendar_dialog_callbacks::{CalendarDialogCallbacks, DeleteCalendarDialogCallbacks};
#[allow(unused_imports)]
pub use event_dialog_callbacks::EventDialogCallbacks;
