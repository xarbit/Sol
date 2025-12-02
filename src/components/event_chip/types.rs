//! Event chip type definitions
//!
//! Core types used across event chip rendering.

use chrono::{NaiveDate, NaiveTime};

/// Position within a multi-day event span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanPosition {
    /// Single-day event (not spanning)
    Single,
    /// First day of a multi-day event
    First,
    /// Middle day(s) of a multi-day event
    Middle,
    /// Last day of a multi-day event
    Last,
}

impl SpanPosition {
    /// Create SpanPosition from start/end boolean flags
    /// Useful when working with (is_event_start, is_event_end) tuples
    pub fn from_start_end(is_start: bool, is_end: bool) -> Self {
        match (is_start, is_end) {
            (true, true) => SpanPosition::Single,
            (true, false) => SpanPosition::First,
            (false, true) => SpanPosition::Last,
            (false, false) => SpanPosition::Middle,
        }
    }
}

/// Calculate border radius for an event chip based on span position.
/// Returns [top-left, top-right, bottom-right, bottom-left] radii.
pub fn span_border_radius(span_position: SpanPosition, radius: f32) -> [f32; 4] {
    match span_position {
        SpanPosition::Single => [radius, radius, radius, radius],
        SpanPosition::First => [radius, 0.0, 0.0, radius],
        SpanPosition::Middle => [0.0, 0.0, 0.0, 0.0],
        SpanPosition::Last => [0.0, radius, radius, 0.0],
    }
}

/// Calculate border radius from start/end boolean flags.
/// Convenience function that combines from_start_end and span_border_radius.
pub fn span_border_radius_from_flags(is_start: bool, is_end: bool, radius: f32) -> [f32; 4] {
    span_border_radius(SpanPosition::from_start_end(is_start, is_end), radius)
}

/// Calculate padding for an event chip based on span position.
/// Returns [top, right, bottom, left] padding.
/// Reduces padding on sides that continue to adjacent days.
pub fn span_padding(span_position: SpanPosition) -> [u16; 4] {
    match span_position {
        SpanPosition::Single => [2, 4, 2, 4],
        SpanPosition::First => [2, 0, 2, 4],   // No right padding - continues right
        SpanPosition::Middle => [2, 0, 2, 0],  // No horizontal padding - continues both sides
        SpanPosition::Last => [2, 4, 2, 0],    // No left padding - continues left
    }
}

/// Opacity values for event chip rendering based on selection/drag state.
/// Used to provide visual feedback when events are selected or being dragged.
#[derive(Debug, Clone, Copy)]
pub struct ChipOpacity {
    /// Background opacity (e.g., 0.15 for dragging, 0.5 for selected, 0.3 default)
    pub background: f32,
    /// Text opacity (e.g., 0.4 for dragging, 1.0 otherwise)
    pub text: f32,
}

/// Multiplier for dimming past events (applied to both background and text)
const PAST_EVENT_DIM_FACTOR: f32 = 0.5;

impl ChipOpacity {
    /// Calculate opacity values based on selection and drag state.
    /// - Dragging: very dim background (0.15), dim text (0.4) to show event is "in flight"
    /// - Selected: semi-transparent background (0.5), full text (1.0)
    /// - Default: subtle background (0.3), full text (1.0)
    pub fn from_state(is_selected: bool, is_being_dragged: bool) -> Self {
        let background = if is_being_dragged {
            0.15
        } else if is_selected {
            0.5
        } else {
            0.3
        };
        let text = if is_being_dragged { 0.4 } else { 1.0 };
        Self { background, text }
    }

    /// Calculate opacity values for past events (dimmed to reduce visual prominence).
    /// Past events are shown with reduced opacity to help focus on upcoming events.
    pub fn from_state_with_past(is_selected: bool, is_being_dragged: bool, is_past: bool) -> Self {
        let mut opacity = Self::from_state(is_selected, is_being_dragged);
        if is_past {
            opacity.background *= PAST_EVENT_DIM_FACTOR;
            opacity.text *= PAST_EVENT_DIM_FACTOR;
        }
        opacity
    }

    /// Calculate opacity for a dot/indicator element during drag.
    /// Dots don't have selection state, only drag state.
    pub fn dot_opacity(is_being_dragged: bool) -> f32 {
        if is_being_dragged { 0.3 } else { 1.0 }
    }

    /// Get background opacity for week/day view timed events.
    /// Returns (background_opacity, border_width) tuple.
    /// All timed events have some transparency for a softer look.
    /// Past events are more transparent to reduce visual prominence.
    pub fn timed_event_opacity(is_selected: bool, is_past: bool) -> (f32, f32) {
        let (base_bg, border) = if is_selected {
            (0.8, 2.0)  // Selected: slightly more opaque
        } else {
            (0.7, 0.0)  // Normal: transparent for softer look
        };
        // Past events get additional dimming (multiply by 0.5)
        let bg = if is_past { base_bg * PAST_EVENT_DIM_FACTOR } else { base_bg };
        (bg, border)
    }
}

/// Selection and drag state for event chips.
/// Pass `Some(ChipSelectionState)` to render with selection/drag support,
/// or `None` for a simple non-interactive chip.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChipSelectionState {
    /// Whether this event is currently selected
    pub is_selected: bool,
    /// Whether this specific event is being dragged
    pub is_being_dragged: bool,
}

impl ChipSelectionState {
    /// Create selection state from individual flags
    pub fn new(is_selected: bool, is_being_dragged: bool) -> Self {
        Self { is_selected, is_being_dragged }
    }
}

/// Event with associated calendar color for display
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    pub calendar_id: String, // Calendar ID for unique identification
    pub uid: String,
    pub summary: String,
    pub color: String,      // Hex color from calendar
    pub all_day: bool,      // Whether this is an all-day event
    pub start_time: Option<NaiveTime>, // Start time for timed events
    pub end_time: Option<NaiveTime>,   // End time for timed events (for week/day view positioning)
    /// Start date of the event span (for multi-day events)
    pub span_start: Option<NaiveDate>,
    /// End date of the event span (for multi-day events)
    pub span_end: Option<NaiveDate>,
}

impl DisplayEvent {
    /// Get a unique identifier for this event that includes the calendar
    /// This ensures events with the same UID from different calendars are treated as distinct
    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.calendar_id, self.uid)
    }

    /// Check if this is a multi-day all-day event
    pub fn is_multi_day(&self) -> bool {
        self.all_day
            && self.span_start.is_some()
            && self.span_end.is_some()
            && self.span_start != self.span_end
    }

    /// Get the span position for a given date within this event
    pub fn span_position_for_date(&self, date: NaiveDate) -> SpanPosition {
        match (self.span_start, self.span_end) {
            (Some(start), Some(end)) if start != end => {
                if date == start {
                    SpanPosition::First
                } else if date == end {
                    SpanPosition::Last
                } else if date > start && date < end {
                    SpanPosition::Middle
                } else {
                    SpanPosition::Single
                }
            }
            _ => SpanPosition::Single,
        }
    }
}
