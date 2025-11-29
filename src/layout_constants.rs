/// Layout and dimension constants for consistent sizing across the application

// Layout dimensions
pub const SIDEBAR_WIDTH: f32 = 280.0;
pub const MINI_CALENDAR_DAY_BUTTON_SIZE: f32 = 32.0;

// Calendar view dimensions
pub const HOUR_ROW_HEIGHT: f32 = 60.0; // Height of each hour slot in week/day views
pub const TIME_LABEL_WIDTH: f32 = 60.0; // Width of the time labels column
pub const ALL_DAY_HEADER_HEIGHT: f32 = 40.0; // Height of all-day events section
pub const WEEK_NUMBER_WIDTH: f32 = 32.0; // Width of week number column (slim)

// Menu bar dimensions (COSMIC standard)
pub const MENU_ITEM_HEIGHT: u16 = 40;
pub const MENU_ITEM_WIDTH: u16 = 240;
pub const MENU_SPACING: f32 = 4.0;

// Spacing values
pub const SPACING_TINY: u16 = 1;
pub const SPACING_XXS: u16 = 2;
pub const SPACING_SMALL: u16 = 4;
pub const SPACING_MEDIUM: u16 = 8;
pub const SPACING_MINI_CALENDAR: u16 = 12;
pub const SPACING_LARGE: u16 = 20;

// Padding values
pub const PADDING_TINY: u16 = 4;
pub const PADDING_SMALL: u16 = 8;
pub const PADDING_MEDIUM: u16 = 12;
pub const PADDING_STANDARD: u16 = 16;
pub const PADDING_MONTH_GRID: u16 = 20;
pub const PADDING_LARGE: u16 = 40;
pub const PADDING_DAY_CELL: [u16; 4] = [4, 8, 0, 0]; // top, right, bottom, left
pub const PADDING_COLOR_PICKER_NESTED: [u16; 4] = [4, 0, 4, 36]; // indented color picker

// Border styling
pub const BORDER_RADIUS: [f32; 4] = [4.0, 4.0, 4.0, 4.0];
pub const BORDER_WIDTH_THIN: f32 = 0.5; // Cell grid borders
pub const BORDER_WIDTH_NORMAL: f32 = 1.0; // Standard day cell borders
pub const BORDER_WIDTH_HIGHLIGHT: f32 = 2.0; // Today indicator and selected items
pub const BORDER_WIDTH_SELECTED: f32 = 3.0; // Color picker selected state
pub const LIGHT_BORDER_OPACITY: f32 = 0.2;

// Color picker dimensions
pub const COLOR_BUTTON_SIZE_SMALL: f32 = 28.0;
pub const COLOR_BUTTON_SIZE_MEDIUM: f32 = 32.0;
pub const COLOR_BUTTON_SIZE_LARGE: f32 = 36.0;
pub const COLOR_INDICATOR_SIZE: f32 = 24.0; // Size of color indicator circle
pub const COLOR_PALETTE_COLUMNS: usize = 6; // Number of colors per row in palette grid
pub const SPACING_COLOR_GRID: u16 = 6;
pub const SPACING_COLOR_CONTAINER: u16 = 8;

// Shadow properties
pub const SHADOW_OPACITY: f32 = 0.3;
pub const SHADOW_OFFSET_X: f32 = 2.0;
pub const SHADOW_OFFSET_Y: f32 = 0.0;
pub const SHADOW_BLUR_RADIUS: f32 = 10.0;

// Icon names
pub const ICON_PREVIOUS: &str = "go-previous-symbolic";
pub const ICON_NEXT: &str = "go-next-symbolic";
pub const ICON_SIDEBAR_OPEN: &str = "navbar-open-symbolic";
pub const ICON_SIDEBAR_CLOSED: &str = "navbar-closed-symbolic";
pub const ICON_SEARCH: &str = "system-search-symbolic";

// Font sizes
#[allow(dead_code)] // For future use
pub const FONT_SIZE_SMALL: u16 = 11;
pub const FONT_SIZE_MEDIUM: u16 = 12;
#[allow(dead_code)] // For future use
pub const FONT_SIZE_BODY: u16 = 14;
pub const FONT_SIZE_LARGE: u16 = 24;
