//! Event display mode calculation for responsive calendar views.
//!
//! This module provides shared logic for determining how events should be
//! displayed based on available cell dimensions. Used by both day cells
//! and the overlay layer to ensure consistent rendering.

use cosmic::iced::Size;

use crate::ui_constants::{
    COMPACT_EVENT_HEIGHT, COMPACT_OVERFLOW_HEIGHT, DATE_EVENT_HEIGHT,
    DAY_HEADER_HEIGHT, MIN_CELL_HEIGHT_FOR_FULL_EVENTS, MIN_CELL_HEIGHT_FOR_OVERFLOW,
    MIN_CELL_WIDTH_FOR_FULL_EVENTS, OVERFLOW_INDICATOR_HEIGHT, PADDING_DAY_CELL,
    SPACING_TINY,
};

/// Vertical-only padding for day cells (derived from PADDING_DAY_CELL)
const PADDING_DAY_CELL_VERTICAL_TOP: f32 = PADDING_DAY_CELL[0] as f32;

/// Spacing between events (derived from SPACING_TINY)
const EVENT_SPACING: f32 = SPACING_TINY as f32;

/// Display mode for events based on available cell size.
///
/// This enum determines how events are rendered in day cells and overlays:
/// - `Full`: Regular event chips with text and colored backgrounds
/// - `Compact`: Thin colored lines without text (for small cells)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventDisplayMode {
    /// Full event chips with text
    Full {
        /// Maximum number of events that can be displayed
        max_visible: usize,
        /// Whether to show the overflow indicator ("+N more")
        show_overflow: bool,
    },
    /// Compact color-only indicators (thin lines without text)
    Compact {
        /// Maximum number of events that can be displayed
        max_visible: usize,
        /// Whether to show the overflow indicator ("+N")
        show_overflow: bool,
    },
}

impl EventDisplayMode {
    /// Returns true if this is compact mode
    pub fn is_compact(&self) -> bool {
        matches!(self, EventDisplayMode::Compact { .. })
    }

    /// Returns the maximum number of visible events
    pub fn max_visible(&self) -> usize {
        match self {
            EventDisplayMode::Full { max_visible, .. } => *max_visible,
            EventDisplayMode::Compact { max_visible, .. } => *max_visible,
        }
    }

    /// Returns whether the overflow indicator should be shown
    pub fn show_overflow(&self) -> bool {
        match self {
            EventDisplayMode::Full { show_overflow, .. } => *show_overflow,
            EventDisplayMode::Compact { show_overflow, .. } => *show_overflow,
        }
    }
}

/// Calculate the event display mode based on cell dimensions.
///
/// This function determines:
/// 1. Whether to use full or compact mode (based on cell size thresholds)
/// 2. How many events can fit in the available space
/// 3. Whether there's room for the overflow indicator
///
/// # Arguments
/// * `cell_size` - The dimensions of the day cell
///
/// # Returns
/// The appropriate `EventDisplayMode` for the given cell size
pub fn calculate_display_mode(cell_size: Size) -> EventDisplayMode {
    let use_compact = cell_size.height < MIN_CELL_HEIGHT_FOR_FULL_EVENTS
        || cell_size.width < MIN_CELL_WIDTH_FOR_FULL_EVENTS;

    // Whether to show overflow indicator at all
    let show_overflow = cell_size.height >= MIN_CELL_HEIGHT_FOR_OVERFLOW;

    // Available height for events (cell height minus header and padding)
    let base_available = (cell_size.height - DAY_HEADER_HEIGHT - (PADDING_DAY_CELL_VERTICAL_TOP * 2.0)).max(0.0);

    if use_compact {
        // Reserve space for overflow indicator if we'll show it
        let overflow_reserve = if show_overflow { COMPACT_OVERFLOW_HEIGHT + EVENT_SPACING } else { 0.0 };
        let available_height = (base_available - overflow_reserve).max(0.0);

        // Compact mode: thin lines
        let max_visible = ((available_height + EVENT_SPACING) / (COMPACT_EVENT_HEIGHT + EVENT_SPACING)).floor() as usize;
        EventDisplayMode::Compact { max_visible: max_visible.max(1), show_overflow }
    } else {
        // Reserve space for overflow indicator if we'll show it
        let overflow_reserve = if show_overflow { OVERFLOW_INDICATOR_HEIGHT + EVENT_SPACING } else { 0.0 };
        let available_height = (base_available - overflow_reserve).max(0.0);

        // Full mode: regular event chips
        let max_visible = ((available_height + EVENT_SPACING) / (DATE_EVENT_HEIGHT + EVENT_SPACING)).floor() as usize;
        EventDisplayMode::Full { max_visible: max_visible.max(1), show_overflow }
    }
}

/// Check if a cell size should use compact mode.
///
/// This is a simpler check for cases where you only need to know
/// compact vs full, not the full display mode details.
///
/// # Arguments
/// * `cell_width` - Width of the cell
/// * `cell_height` - Height of the cell
///
/// # Returns
/// `true` if compact mode should be used
pub fn should_use_compact(cell_width: f32, cell_height: f32) -> bool {
    cell_height < MIN_CELL_HEIGHT_FOR_FULL_EVENTS
        || cell_width < MIN_CELL_WIDTH_FOR_FULL_EVENTS
}
