//! Selection state management for multi-day/time event creation and event dragging.
//!
//! This module provides state tracking for:
//! - Drag selection across day/time cells for creating new events
//! - Event drag-and-drop for moving existing events
//!
//! The architecture supports both date-only (month view) and date+time (week/day views) operations.
//!
//! # Usage Flow for Selection
//!
//! ```text
//! User presses on day/time cell
//!         │
//!         ▼
//! Message::SelectionStart(date, time)
//!         │
//!         ▼
//! SelectionState { start: (date, time), end: (date, time), active: true }
//!         │
//!         ▼
//! User drags over other cells
//!         │
//!         ▼
//! Message::SelectionUpdate(date, time)  [for each cell entered]
//!         │
//!         ▼
//! SelectionState { start: original, end: current, active: true }
//!         │
//!         ▼
//! User releases mouse
//!         │
//!         ▼
//! Message::SelectionEnd
//!         │
//!         ├── Month view: Date range → Quick event or event dialog
//!         └── Week/Day view: Time range → Event dialog with times pre-filled
//! ```

use chrono::{NaiveDate, NaiveTime};
use log::debug;

/// A point in time that may or may not include a specific time.
/// - `time = None` means "all day" or "date only" (month view)
/// - `time = Some(t)` means a specific time (week/day views)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPoint {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
}

impl SelectionPoint {
    /// Create a date-only selection point (for month view)
    pub fn date_only(date: NaiveDate) -> Self {
        Self { date, time: None }
    }

    /// Create a date+time selection point (for week/day views)
    pub fn with_time(date: NaiveDate, time: NaiveTime) -> Self {
        Self { date, time: Some(time) }
    }

    /// Get the date component
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    /// Get the time component (if any)
    pub fn time(&self) -> Option<NaiveTime> {
        self.time
    }

    /// Check if this is a date-only point
    pub fn is_date_only(&self) -> bool {
        self.time.is_none()
    }
}

/// State for tracking drag selection across day/time cells.
///
/// This is a transient UI state, not a dialog, so it lives directly
/// in CosmicCalendar rather than in ActiveDialog.
///
/// Supports both:
/// - Date-only selection (month view): Uses SelectionPoint with time = None
/// - Time-based selection (week/day views): Uses SelectionPoint with time = Some(t)
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// The point where the selection started (mouse press)
    start: Option<SelectionPoint>,
    /// The current end point of the selection (follows mouse)
    end: Option<SelectionPoint>,
    /// Whether a drag selection is currently active
    pub is_active: bool,
}

impl SelectionState {
    /// Create a new empty selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new date-only selection (for month view)
    pub fn start(&mut self, date: NaiveDate) {
        self.start_at(SelectionPoint::date_only(date));
    }

    /// Start a new time-based selection (for week/day views)
    pub fn start_with_time(&mut self, date: NaiveDate, time: NaiveTime) {
        self.start_at(SelectionPoint::with_time(date, time));
    }

    /// Start a new selection at the given point
    fn start_at(&mut self, point: SelectionPoint) {
        debug!("SelectionState: Starting selection at {:?}", point);
        self.start = Some(point);
        self.end = Some(point);
        self.is_active = true;
    }

    /// Update the selection end point with date only (for month view)
    pub fn update(&mut self, date: NaiveDate) {
        self.update_to(SelectionPoint::date_only(date));
    }

    /// Update the selection end point with date and time (for week/day views)
    pub fn update_with_time(&mut self, date: NaiveDate, time: NaiveTime) {
        self.update_to(SelectionPoint::with_time(date, time));
    }

    /// Update the selection end point
    fn update_to(&mut self, point: SelectionPoint) {
        if self.is_active {
            debug!("SelectionState: Updating selection end to {:?}", point);
            self.end = Some(point);
        }
    }

    /// End the selection and return the selected range if valid
    pub fn end(&mut self) -> Option<SelectionRange> {
        if !self.is_active {
            return None;
        }

        let range = self.get_range();
        debug!("SelectionState: Ending selection with range {:?}", range);
        self.reset();
        range
    }

    /// Cancel the current selection
    pub fn cancel(&mut self) {
        debug!("SelectionState: Cancelling selection");
        self.reset();
    }

    /// Reset the selection state
    pub fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.is_active = false;
    }

    /// Get the current selection range (normalized so start <= end)
    pub fn get_range(&self) -> Option<SelectionRange> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some(SelectionRange::new(start, end)),
            _ => None,
        }
    }

    /// Check if a date is within the current selection (ignoring time)
    pub fn contains(&self, date: NaiveDate) -> bool {
        self.get_range()
            .map(|r| r.contains_date(date))
            .unwrap_or(false)
    }

    /// Check if the current selection spans multiple days
    pub fn is_multi_day(&self) -> bool {
        self.get_range()
            .map(|r| r.is_multi_day())
            .unwrap_or(false)
    }

    /// Get the start date (for backwards compatibility)
    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start.map(|p| p.date)
    }

    /// Get the end date (for backwards compatibility)
    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end.map(|p| p.date)
    }
}

/// A normalized selection range with start <= end.
/// Supports both date-only and date+time ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    /// The start point of the selection
    pub start: SelectionPoint,
    /// The end point of the selection
    pub end: SelectionPoint,
}

impl SelectionRange {
    /// Create a new range, normalizing so start <= end
    pub fn new(point1: SelectionPoint, point2: SelectionPoint) -> Self {
        // Compare by date first, then by time
        let p1_key = (point1.date, point1.time);
        let p2_key = (point2.date, point2.time);

        if p1_key <= p2_key {
            Self { start: point1, end: point2 }
        } else {
            Self { start: point2, end: point1 }
        }
    }

    /// Create a date-only range (for month view compatibility)
    pub fn from_dates(date1: NaiveDate, date2: NaiveDate) -> Self {
        Self::new(SelectionPoint::date_only(date1), SelectionPoint::date_only(date2))
    }

    /// Get the start date
    pub fn start_date(&self) -> NaiveDate {
        self.start.date
    }

    /// Get the end date
    pub fn end_date(&self) -> NaiveDate {
        self.end.date
    }

    /// Get the start time (if any)
    pub fn start_time(&self) -> Option<NaiveTime> {
        self.start.time
    }

    /// Get the end time (if any)
    pub fn end_time(&self) -> Option<NaiveTime> {
        self.end.time
    }

    /// Check if this is a date-only range (no times specified)
    pub fn is_date_only(&self) -> bool {
        self.start.is_date_only() && self.end.is_date_only()
    }

    /// Check if a date is within this range (inclusive, ignoring time)
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start.date && date <= self.end.date
    }

    /// Check if this range spans multiple days
    pub fn is_multi_day(&self) -> bool {
        self.start.date != self.end.date
    }

    /// Get the number of days in this range
    pub fn day_count(&self) -> i64 {
        (self.end.date - self.start.date).num_days() + 1
    }

    /// Iterate over all dates in this range
    pub fn dates(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start.date;
        let end = self.end.date;
        (0..=((end - start).num_days())).map(move |i| start + chrono::Duration::days(i))
    }
}

// === Event Drag State ===

/// Display information for the drag preview.
/// Separated from EventDragState to maintain clean architecture.
#[derive(Debug, Clone, Default)]
pub struct DragPreviewInfo {
    /// Event summary for the drag preview
    pub summary: Option<String>,
    /// Event color (hex) for the drag preview
    pub color: Option<String>,
    /// Current cursor position for rendering drag preview (x, y)
    pub cursor_position: Option<(f32, f32)>,
}

impl DragPreviewInfo {
    /// Create a new empty preview info
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event display info
    pub fn set_event_info(&mut self, summary: String, color: String) {
        self.summary = Some(summary);
        self.color = Some(color);
    }

    /// Update cursor position
    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.cursor_position = Some((x, y));
    }

    /// Reset the preview info
    pub fn reset(&mut self) {
        self.summary = None;
        self.color = None;
        self.cursor_position = None;
    }
}

/// Target location for an event drag operation.
/// Supports both date-only (month view) and date+time (week/day views).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragTarget {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
}

impl DragTarget {
    /// Create a date-only target (for month view)
    pub fn date_only(date: NaiveDate) -> Self {
        Self { date, time: None }
    }

    /// Create a date+time target (for week/day views)
    pub fn with_time(date: NaiveDate, time: NaiveTime) -> Self {
        Self { date, time: Some(time) }
    }
}

/// State for tracking event drag-and-drop to move events to a new date/time.
///
/// This is separate from SelectionState which is for creating new multi-day events.
/// EventDragState tracks dragging an existing event to a new location.
///
/// Display concerns (preview rendering) are separated into DragPreviewInfo.
#[derive(Debug, Clone, Default)]
pub struct EventDragState {
    /// The UID of the event being dragged
    pub event_uid: Option<String>,
    /// The original start date of the event
    pub original_date: Option<NaiveDate>,
    /// The original start time of the event (if it's a timed event)
    pub original_time: Option<NaiveTime>,
    /// The current target location (where the event would be dropped)
    target: Option<DragTarget>,
    /// Whether a drag operation is currently active
    pub is_active: bool,
    /// Display information for the drag preview (separated concern)
    pub preview: DragPreviewInfo,
}

impl EventDragState {
    /// Create a new empty drag state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start dragging an event (date-only, for month view)
    pub fn start(&mut self, event_uid: String, original_date: NaiveDate, summary: String, color: String) {
        self.start_internal(event_uid, original_date, None, summary, color);
    }

    /// Start dragging an event with time (for week/day views)
    pub fn start_with_time(&mut self, event_uid: String, original_date: NaiveDate, original_time: NaiveTime, summary: String, color: String) {
        self.start_internal(event_uid, original_date, Some(original_time), summary, color);
    }

    /// Internal start implementation
    fn start_internal(&mut self, event_uid: String, original_date: NaiveDate, original_time: Option<NaiveTime>, summary: String, color: String) {
        debug!("EventDragState: Starting drag for event {} from {} {:?}", event_uid, original_date, original_time);
        self.event_uid = Some(event_uid);
        self.original_date = Some(original_date);
        self.original_time = original_time;
        self.target = Some(DragTarget { date: original_date, time: original_time });
        self.is_active = true;
        self.preview.set_event_info(summary, color);
    }

    /// Update cursor position during drag
    pub fn update_cursor(&mut self, x: f32, y: f32) {
        if self.is_active {
            self.preview.update_cursor(x, y);
        }
    }

    /// Update the target date during drag (date-only, for month view)
    pub fn update(&mut self, target_date: NaiveDate) {
        if self.is_active {
            debug!("EventDragState: Updating target to {}", target_date);
            self.target = Some(DragTarget::date_only(target_date));
        }
    }

    /// Update the target date and time during drag (for week/day views)
    pub fn update_with_time(&mut self, target_date: NaiveDate, target_time: NaiveTime) {
        if self.is_active {
            debug!("EventDragState: Updating target to {} {:?}", target_date, target_time);
            self.target = Some(DragTarget::with_time(target_date, target_time));
        }
    }

    /// End the drag operation and return the move details if valid
    /// Returns (event_uid, original_date, new_date) if a move should occur
    /// For time-aware moves, use end_with_time()
    pub fn end(&mut self) -> Option<(String, NaiveDate, NaiveDate)> {
        if !self.is_active {
            return None;
        }

        let result = match (&self.event_uid, self.original_date, self.target) {
            (Some(uid), Some(original), Some(target)) if original != target.date => {
                debug!("EventDragState: Ending drag - move {} from {} to {}", uid, original, target.date);
                Some((uid.clone(), original, target.date))
            }
            _ => {
                debug!("EventDragState: Ending drag - no move (same date or invalid)");
                None
            }
        };

        self.reset();
        result
    }

    /// End the drag operation with full time information
    /// Returns (event_uid, original_date, original_time, new_date, new_time) if a move should occur
    pub fn end_with_time(&mut self) -> Option<(String, NaiveDate, Option<NaiveTime>, NaiveDate, Option<NaiveTime>)> {
        if !self.is_active {
            return None;
        }

        let result = match (&self.event_uid, self.original_date, self.target) {
            (Some(uid), Some(original_date), Some(target)) => {
                let has_change = original_date != target.date || self.original_time != target.time;
                if has_change {
                    debug!("EventDragState: Ending drag - move {} from {} {:?} to {} {:?}",
                           uid, original_date, self.original_time, target.date, target.time);
                    Some((uid.clone(), original_date, self.original_time, target.date, target.time))
                } else {
                    debug!("EventDragState: Ending drag - no move (same location)");
                    None
                }
            }
            _ => {
                debug!("EventDragState: Ending drag - invalid state");
                None
            }
        };

        self.reset();
        result
    }

    /// Cancel the drag operation
    pub fn cancel(&mut self) {
        debug!("EventDragState: Cancelling drag");
        self.reset();
    }

    /// Reset the drag state
    pub fn reset(&mut self) {
        self.event_uid = None;
        self.original_date = None;
        self.original_time = None;
        self.target = None;
        self.is_active = false;
        self.preview.reset();
    }

    /// Get the target date (if any)
    pub fn target_date(&self) -> Option<NaiveDate> {
        self.target.map(|t| t.date)
    }

    /// Get the target time (if any)
    pub fn target_time(&self) -> Option<NaiveTime> {
        self.target.and_then(|t| t.time)
    }

    /// Get the date offset (number of days to move)
    pub fn get_offset(&self) -> Option<i64> {
        match (self.original_date, self.target) {
            (Some(original), Some(target)) => Some((target.date - original).num_days()),
            _ => None,
        }
    }

    // === Accessors for preview info (for backwards compatibility) ===

    /// Get the event summary for preview
    pub fn event_summary(&self) -> Option<&str> {
        self.preview.summary.as_deref()
    }

    /// Get the event color for preview
    pub fn event_color(&self) -> Option<&str> {
        self.preview.color.as_deref()
    }

    /// Get the cursor position for preview
    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.preview.cursor_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_point_date_only() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let point = SelectionPoint::date_only(date);

        assert_eq!(point.date(), date);
        assert!(point.time().is_none());
        assert!(point.is_date_only());
    }

    #[test]
    fn test_selection_point_with_time() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let point = SelectionPoint::with_time(date, time);

        assert_eq!(point.date(), date);
        assert_eq!(point.time(), Some(time));
        assert!(!point.is_date_only());
    }

    #[test]
    fn test_selection_state_start() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start(date);

        assert!(state.is_active);
        assert_eq!(state.start_date(), Some(date));
        assert_eq!(state.end_date(), Some(date));
    }

    #[test]
    fn test_selection_state_start_with_time() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        state.start_with_time(date, time);

        assert!(state.is_active);
        let range = state.get_range().unwrap();
        assert_eq!(range.start_time(), Some(time));
    }

    #[test]
    fn test_selection_state_update() {
        let mut state = SelectionState::new();
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start(start);
        state.update(end);

        assert!(state.is_active);
        assert_eq!(state.start_date(), Some(start));
        assert_eq!(state.end_date(), Some(end));
    }

    #[test]
    fn test_selection_state_end() {
        let mut state = SelectionState::new();
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start(start);
        state.update(end);
        let range = state.end();

        assert!(!state.is_active);
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start_date(), start);
        assert_eq!(range.end_date(), end);
    }

    #[test]
    fn test_selection_range_normalized() {
        let early = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let late = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        // Forward selection
        let range1 = SelectionRange::from_dates(early, late);
        assert_eq!(range1.start_date(), early);
        assert_eq!(range1.end_date(), late);

        // Backward selection (dragging up/left)
        let range2 = SelectionRange::from_dates(late, early);
        assert_eq!(range2.start_date(), early);
        assert_eq!(range2.end_date(), late);
    }

    #[test]
    fn test_selection_range_contains() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let range = SelectionRange::from_dates(start, end);

        assert!(range.contains_date(start));
        assert!(range.contains_date(end));
        assert!(range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()));
        assert!(!range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()));
        assert!(!range.contains_date(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()));
    }

    #[test]
    fn test_selection_range_is_multi_day() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let single = SelectionRange::from_dates(date1, date1);
        assert!(!single.is_multi_day());

        let multi = SelectionRange::from_dates(date1, date2);
        assert!(multi.is_multi_day());
    }

    #[test]
    fn test_selection_range_day_count() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let range = SelectionRange::from_dates(start, end);

        assert_eq!(range.day_count(), 6); // 10, 11, 12, 13, 14, 15
    }

    #[test]
    fn test_selection_range_dates_iterator() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        let range = SelectionRange::from_dates(start, end);

        let dates: Vec<_> = range.dates().collect();
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2024, 1, 11).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2024, 1, 12).unwrap());
    }

    #[test]
    fn test_selection_state_cancel() {
        let mut state = SelectionState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start(date);
        assert!(state.is_active);

        state.cancel();
        assert!(!state.is_active);
        assert!(state.start_date().is_none());
        assert!(state.end_date().is_none());
    }

    // EventDragState tests

    #[test]
    fn test_event_drag_state_start() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());

        assert!(state.is_active);
        assert_eq!(state.event_uid, Some("event-123".to_string()));
        assert_eq!(state.original_date, Some(date));
        assert_eq!(state.target_date(), Some(date));
        assert_eq!(state.event_summary(), Some("Test Event"));
        assert_eq!(state.event_color(), Some("#0000ff"));
    }

    #[test]
    fn test_event_drag_state_start_with_time() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        state.start_with_time("event-123".to_string(), date, time, "Test Event".to_string(), "#0000ff".to_string());

        assert!(state.is_active);
        assert_eq!(state.original_time, Some(time));
        assert_eq!(state.target_time(), Some(time));
    }

    #[test]
    fn test_event_drag_state_update() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);

        assert!(state.is_active);
        assert_eq!(state.original_date, Some(original));
        assert_eq!(state.target_date(), Some(target));
    }

    #[test]
    fn test_event_drag_state_end_with_move() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);
        let result = state.end();

        assert!(!state.is_active);
        assert!(result.is_some());
        let (uid, orig, tgt) = result.unwrap();
        assert_eq!(uid, "event-123");
        assert_eq!(orig, original);
        assert_eq!(tgt, target);
    }

    #[test]
    fn test_event_drag_state_end_same_date() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());
        // Don't update - target stays same as original
        let result = state.end();

        assert!(!state.is_active);
        assert!(result.is_none()); // No move needed
    }

    #[test]
    fn test_event_drag_state_end_with_time() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let original_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let target_time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();

        state.start_with_time("event-123".to_string(), date, original_time, "Test Event".to_string(), "#0000ff".to_string());
        state.update_with_time(date, target_time);
        let result = state.end_with_time();

        assert!(!state.is_active);
        assert!(result.is_some());
        let (uid, orig_date, orig_time, tgt_date, tgt_time) = result.unwrap();
        assert_eq!(uid, "event-123");
        assert_eq!(orig_date, date);
        assert_eq!(orig_time, Some(original_time));
        assert_eq!(tgt_date, date);
        assert_eq!(tgt_time, Some(target_time));
    }

    #[test]
    fn test_event_drag_state_get_offset() {
        let mut state = EventDragState::new();
        let original = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        state.start("event-123".to_string(), original, "Test Event".to_string(), "#0000ff".to_string());
        state.update(target);

        assert_eq!(state.get_offset(), Some(3)); // 3 days forward
    }

    #[test]
    fn test_event_drag_state_cancel() {
        let mut state = EventDragState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        state.start("event-123".to_string(), date, "Test Event".to_string(), "#0000ff".to_string());
        assert!(state.is_active);

        state.cancel();
        assert!(!state.is_active);
        assert!(state.event_uid.is_none());
        assert!(state.event_summary().is_none());
        assert!(state.event_color().is_none());
    }

    #[test]
    fn test_drag_preview_info() {
        let mut preview = DragPreviewInfo::new();

        preview.set_event_info("Test".to_string(), "#ff0000".to_string());
        assert_eq!(preview.summary, Some("Test".to_string()));
        assert_eq!(preview.color, Some("#ff0000".to_string()));

        preview.update_cursor(100.0, 200.0);
        assert_eq!(preview.cursor_position, Some((100.0, 200.0)));

        preview.reset();
        assert!(preview.summary.is_none());
        assert!(preview.color.is_none());
        assert!(preview.cursor_position.is_none());
    }
}
