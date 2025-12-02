//! Event drag state for moving events via drag-and-drop.

use chrono::{NaiveDate, NaiveTime};
use log::debug;

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
    #[allow(dead_code)] // Part of drag preview API
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
    #[allow(dead_code)] // Reserved for week/day view event dragging
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
    /// The calendar ID of the event being dragged
    pub calendar_id: Option<String>,
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
    pub fn start(&mut self, calendar_id: String, event_uid: String, original_date: NaiveDate, summary: String, color: String) {
        self.start_internal(calendar_id, event_uid, original_date, None, summary, color);
    }

    /// Start dragging an event with time (for week/day views)
    #[allow(dead_code)] // Reserved for week/day view event dragging
    pub fn start_with_time(&mut self, calendar_id: String, event_uid: String, original_date: NaiveDate, original_time: NaiveTime, summary: String, color: String) {
        self.start_internal(calendar_id, event_uid, original_date, Some(original_time), summary, color);
    }

    /// Internal start implementation
    fn start_internal(&mut self, calendar_id: String, event_uid: String, original_date: NaiveDate, original_time: Option<NaiveTime>, summary: String, color: String) {
        debug!("EventDragState: Starting drag for calendar={} event={} from {} {:?}", calendar_id, event_uid, original_date, original_time);
        self.calendar_id = Some(calendar_id);
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
    #[allow(dead_code)] // Reserved for week/day view event dragging
    pub fn update_with_time(&mut self, target_date: NaiveDate, target_time: NaiveTime) {
        if self.is_active {
            debug!("EventDragState: Updating target to {} {:?}", target_date, target_time);
            self.target = Some(DragTarget::with_time(target_date, target_time));
        }
    }

    /// End the drag operation and return the move details if valid
    /// Returns (calendar_id, event_uid, original_date, new_date) if a move should occur
    /// For time-aware moves, use end_with_time()
    pub fn end(&mut self) -> Option<(String, String, NaiveDate, NaiveDate)> {
        if !self.is_active {
            return None;
        }

        let result = match (&self.calendar_id, &self.event_uid, self.original_date, self.target) {
            (Some(cal_id), Some(uid), Some(original), Some(target)) if original != target.date => {
                debug!("EventDragState: Ending drag - move calendar={} event={} from {} to {}", cal_id, uid, original, target.date);
                Some((cal_id.clone(), uid.clone(), original, target.date))
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
    #[allow(dead_code)] // Reserved for week/day view event dragging with time
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
        self.calendar_id = None;
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
    #[allow(dead_code)] // Reserved for week/day view time-based operations
    pub fn target_time(&self) -> Option<NaiveTime> {
        self.target.and_then(|t| t.time)
    }

    /// Get the date offset (number of days to move)
    #[allow(dead_code)] // Reserved for multi-day event dragging
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
