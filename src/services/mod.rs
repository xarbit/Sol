//! Services layer for business logic and middleware.
//!
//! This module contains service handlers that sit between the UI/update layer
//! and the protocol/storage layer. Each handler centralizes operations for
//! a specific domain:
//!
//! - `EventHandler` - Event CRUD operations (create, read, update, delete events)
//! - `CalendarHandler` - Calendar management (create, edit, delete calendars)
//! - `SettingsHandler` - Application settings (load, save, validate settings)
//! - `SyncHandler` - Synchronization (sync calendars with backends)
//! - `ExportHandler` - Import/Export (iCalendar import/export)

mod calendar_handler;
mod event_handler;
mod export_handler;
mod settings_handler;
mod sync_handler;

pub use calendar_handler::{CalendarHandler, CalendarError, CalendarResult, NewCalendarData, UpdateCalendarData};
pub use event_handler::{EventHandler, EventError, EventResult};
pub use export_handler::{ExportHandler, ExportError, ExportResult};
pub use settings_handler::{SettingsHandler, SettingsError, SettingsResult};
pub use sync_handler::{SyncHandler, SyncError, SyncResult, SyncReport, CalendarSyncStatus};
