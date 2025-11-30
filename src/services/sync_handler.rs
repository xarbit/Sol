//! Sync Handler - Centralized synchronization management.
//!
//! This handler manages synchronization of calendars with their backends,
//! including local database refreshes and remote CalDAV syncs.

use crate::calendars::CalendarManager;
use std::error::Error;

/// Result type for sync operations
pub type SyncResult<T> = Result<T, SyncError>;

/// Error types for sync operations
#[derive(Debug)]
pub enum SyncError {
    /// Calendar not found
    CalendarNotFound(String),
    /// Sync failed for a calendar
    SyncFailed { calendar_id: String, reason: String },
    /// Multiple sync failures
    MultipleFailed(Vec<(String, String)>),
    /// Network error
    NetworkError(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::CalendarNotFound(id) => write!(f, "Calendar not found: {}", id),
            SyncError::SyncFailed { calendar_id, reason } => {
                write!(f, "Sync failed for {}: {}", calendar_id, reason)
            }
            SyncError::MultipleFailed(failures) => {
                write!(f, "Multiple calendars failed to sync: ")?;
                for (id, reason) in failures {
                    write!(f, "[{}: {}] ", id, reason)?;
                }
                Ok(())
            }
            SyncError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl Error for SyncError {}

/// Sync status for a calendar
#[derive(Debug, Clone)]
pub struct CalendarSyncStatus {
    pub calendar_id: String,
    pub calendar_name: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Overall sync result
#[derive(Debug)]
pub struct SyncReport {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub statuses: Vec<CalendarSyncStatus>,
}

impl SyncReport {
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }
}

/// Sync Handler - centralized synchronization management.
pub struct SyncHandler;

impl SyncHandler {
    /// Sync a single calendar
    pub fn sync_calendar(manager: &mut CalendarManager, calendar_id: &str) -> SyncResult<()> {
        let calendar = manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| SyncError::CalendarNotFound(calendar_id.to_string()))?;

        calendar.sync().map_err(|e| SyncError::SyncFailed {
            calendar_id: calendar_id.to_string(),
            reason: e.to_string(),
        })
    }

    /// Sync all enabled calendars
    pub fn sync_all(manager: &mut CalendarManager) -> SyncReport {
        let mut statuses = Vec::new();
        let mut succeeded = 0;
        let mut failed = 0;

        for calendar in manager.sources_mut().iter_mut() {
            if !calendar.is_enabled() {
                continue;
            }

            let calendar_id = calendar.info().id.clone();
            let calendar_name = calendar.info().name.clone();

            match calendar.sync() {
                Ok(()) => {
                    succeeded += 1;
                    statuses.push(CalendarSyncStatus {
                        calendar_id,
                        calendar_name,
                        success: true,
                        error_message: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    statuses.push(CalendarSyncStatus {
                        calendar_id,
                        calendar_name,
                        success: false,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        SyncReport {
            total: succeeded + failed,
            succeeded,
            failed,
            statuses,
        }
    }

    /// Sync all calendars and return error if any failed
    pub fn sync_all_or_fail(manager: &mut CalendarManager) -> SyncResult<()> {
        let report = Self::sync_all(manager);

        if report.all_succeeded() {
            Ok(())
        } else {
            let failures: Vec<(String, String)> = report
                .statuses
                .into_iter()
                .filter(|s| !s.success)
                .map(|s| (s.calendar_id, s.error_message.unwrap_or_default()))
                .collect();

            Err(SyncError::MultipleFailed(failures))
        }
    }

    /// Check if any calendar requires network for sync
    pub fn has_remote_calendars(manager: &CalendarManager) -> bool {
        manager.sources().iter().any(|c| {
            // Check calendar type - CalDAV calendars require network
            matches!(
                c.info().calendar_type,
                crate::calendars::CalendarType::CalDav
                    | crate::calendars::CalendarType::Google
                    | crate::calendars::CalendarType::Outlook
                    | crate::calendars::CalendarType::ICloud
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_report_all_succeeded() {
        let report = SyncReport {
            total: 2,
            succeeded: 2,
            failed: 0,
            statuses: vec![],
        };
        assert!(report.all_succeeded());
    }

    #[test]
    fn test_sync_report_has_failures() {
        let report = SyncReport {
            total: 2,
            succeeded: 1,
            failed: 1,
            statuses: vec![],
        };
        assert!(!report.all_succeeded());
    }
}
