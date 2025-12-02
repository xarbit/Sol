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

pub use calendar_handler::{CalendarHandler, NewCalendarData, UpdateCalendarData};
pub use event_handler::EventHandler;
pub use export_handler::ExportHandler;
pub use settings_handler::SettingsHandler;

// Internal types - exported for potential future use but not currently needed externally
#[allow(unused_imports)]
pub(crate) use calendar_handler::{CalendarError, CalendarResult};
#[allow(unused_imports)]
pub(crate) use event_handler::{EventError, EventResult};
#[allow(unused_imports)]
pub(crate) use export_handler::{ExportError, ExportResult};
#[allow(unused_imports)]
pub(crate) use settings_handler::{SettingsError, SettingsResult};
#[allow(unused_imports)]
pub(crate) use sync_handler::{SyncHandler, SyncError, SyncResult, SyncReport, CalendarSyncStatus};
