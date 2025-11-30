use chrono::NaiveDate;
use crate::app::EventDialogField;
use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
use crate::dialogs::DialogAction;
use crate::views::CalendarView;
use cosmic::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    // Dialog management (centralized)
    /// Handle a dialog action (open, close, update)
    Dialog(DialogAction),
    /// Close any open dialog (Escape key)
    CloseDialog,

    // View navigation
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(i32, u32, u32), // (year, month, day)
    /// Select a day without navigating away from current month view (for adjacent month days)
    SelectDayNoNavigate(NaiveDate),

    // UI state
    ToggleSidebar,
    /// Triggered on window resize to sync sidebar with condensed state
    WindowResized,
    ToggleSearch,
    ToggleWeekNumbers,

    // Calendar management
    ToggleCalendar(String),
    /// Select a calendar as the active calendar for new events
    SelectCalendar(String),
    /// Toggle the color picker for a calendar (open if closed, close if open)
    ToggleColorPicker(String),
    /// Close the color picker (when clicking outside)
    CloseColorPicker,
    /// Change a calendar's color
    ChangeCalendarColor(String, String),
    /// Open the calendar dialog in Create mode
    OpenNewCalendarDialog,
    /// Open the calendar dialog in Edit mode for a specific calendar
    OpenEditCalendarDialog(String),
    /// Edit calendar by index (from context menu)
    EditCalendarByIndex(usize),
    /// Update calendar name while typing in dialog
    CalendarDialogNameChanged(String),
    /// Update calendar color selection in dialog
    CalendarDialogColorChanged(String),
    /// Confirm the calendar dialog (Create or Edit)
    ConfirmCalendarDialog,
    /// Cancel the calendar dialog
    CancelCalendarDialog,
    /// Delete the currently selected calendar (with confirmation)
    DeleteSelectedCalendar,
    /// Request to delete a specific calendar by ID (opens confirmation dialog)
    RequestDeleteCalendar(String),
    /// Select calendar by index (from context menu)
    SelectCalendarByIndex(usize),
    /// Delete calendar by index (from context menu)
    DeleteCalendarByIndex(usize),
    /// Confirm calendar deletion
    ConfirmDeleteCalendar,
    /// Cancel calendar deletion
    CancelDeleteCalendar,

    // Selection - Drag selection for multi-day events
    /// Start a drag selection at the given date (mouse press)
    SelectionStart(NaiveDate),
    /// Update the selection end point (mouse move while dragging)
    SelectionUpdate(NaiveDate),
    /// End the selection (mouse release) - opens event dialog if multi-day
    SelectionEnd,
    /// Cancel the current selection
    SelectionCancel,

    // Event management - Quick events
    /// Start creating a quick event on a specific date
    StartQuickEvent(NaiveDate),
    /// Update the quick event text while editing
    QuickEventTextChanged(String),
    /// Commit the quick event (on Enter press)
    CommitQuickEvent,
    /// Cancel quick event editing (on Escape or click outside)
    CancelQuickEvent,
    /// Delete an event by its UID
    DeleteEvent(String),
    /// Select an event (single click) for viewing/editing
    SelectEvent(String),

    // Event drag-and-drop
    /// Start dragging an event to move it (uid, original_date, summary, color)
    DragEventStart(String, NaiveDate, String, String),
    /// Update the drag target date as user drags over cells
    DragEventUpdate(NaiveDate),
    /// Update cursor position during drag (x, y)
    DragEventCursorMove(f32, f32),
    /// End the drag operation (drop the event)
    DragEventEnd,
    /// Cancel the drag operation
    DragEventCancel,

    // Event management - Event dialog
    /// Open the event dialog for creating a new event
    OpenNewEventDialog,
    /// Open the event dialog for editing an existing event
    OpenEditEventDialog(String),
    /// Toggle edit mode for a specific field
    EventDialogToggleEdit(EventDialogField, bool),
    /// Update event title in dialog
    EventDialogTitleChanged(String),
    /// Update event location in dialog
    EventDialogLocationChanged(String),
    /// Toggle all-day event in dialog
    EventDialogAllDayToggled(bool),
    /// Update event start date input text
    EventDialogStartDateInputChanged(String),
    /// Update event start date in dialog
    EventDialogStartDateChanged(NaiveDate),
    /// Toggle start date calendar picker
    EventDialogToggleStartDatePicker,
    /// Navigate start date calendar to previous month
    EventDialogStartDateCalendarPrev,
    /// Navigate start date calendar to next month
    EventDialogStartDateCalendarNext,
    /// Toggle start time picker
    EventDialogToggleStartTimePicker,
    /// Set start time hour
    EventDialogStartTimeHourChanged(u32),
    /// Set start time minute
    EventDialogStartTimeMinuteChanged(u32),
    /// Update event end date input text
    EventDialogEndDateInputChanged(String),
    /// Update event end date in dialog
    EventDialogEndDateChanged(NaiveDate),
    /// Toggle end date calendar picker
    EventDialogToggleEndDatePicker,
    /// Navigate end date calendar to previous month
    EventDialogEndDateCalendarPrev,
    /// Navigate end date calendar to next month
    EventDialogEndDateCalendarNext,
    /// Toggle end time picker
    EventDialogToggleEndTimePicker,
    /// Set end time hour
    EventDialogEndTimeHourChanged(u32),
    /// Set end time minute
    EventDialogEndTimeMinuteChanged(u32),
    /// Update travel time in dialog
    EventDialogTravelTimeChanged(TravelTime),
    /// Update repeat frequency in dialog
    EventDialogRepeatChanged(RepeatFrequency),
    /// Update selected calendar in dialog
    EventDialogCalendarChanged(String),
    /// Update invitee input text
    EventDialogInviteeInputChanged(String),
    /// Add an invitee to the list
    EventDialogAddInvitee,
    /// Remove an invitee from the list
    EventDialogRemoveInvitee(usize),
    /// Update alert setting in dialog
    EventDialogAlertChanged(AlertTime),
    /// Update second alert setting in dialog
    EventDialogAlertSecondChanged(Option<AlertTime>),
    /// Add an attachment
    EventDialogAddAttachment(String),
    /// Remove an attachment
    EventDialogRemoveAttachment(usize),
    /// Update URL in dialog
    EventDialogUrlChanged(String),
    /// Handle text editor action for notes
    EventDialogNotesAction(text_editor::Action),
    /// Confirm the event dialog (Create or Save)
    ConfirmEventDialog,
    /// Cancel the event dialog
    CancelEventDialog,

    // Mini calendar
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,

    // Menu actions
    NewEvent,
    ImportICal,
    ExportICal,
    Settings,
    About,
    LaunchUrl(String),
    ToggleContextDrawer,
    Surface(cosmic::surface::Action),
}
