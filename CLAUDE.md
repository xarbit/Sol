# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build
cargo build --release

# Run
cargo run --release

# Check for compilation errors without building
cargo check

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

Always use `--release` flag for performance testing.

## Architecture

Sol is a calendar application for the COSMIC desktop built with libcosmic (iced-based). It follows the **Elm/MVU (Model-View-Update)** pattern:

### Core Flow
- `app.rs` - Main application state (`CosmicCalendar` struct) implementing `cosmic::Application` trait
- `message.rs` - All application messages as an enum
- `update/` - Message handling split by domain (navigation, calendar, event, selection)
- `keyboard.rs` - Centralized keyboard shortcuts (single source of truth)
- `selection.rs` - Drag selection state for dates and time slots

### State Organization
- `models/` - View-specific state structs (`CalendarState`, `WeekState`, `DayState`, `YearState`)
- `cache.rs` - `CalendarCache` pre-computes calendar states for performance
- `settings.rs` - Persistent app settings

### View Layer
- `views/` - Pure rendering functions (take state, return `Element`)
- `components/` - Reusable UI widgets (day_cell, mini_calendar, toolbar, etc.)
- `layout.rs` - Responsive layout management

### Calendar Backend
- `calendars/calendar_source.rs` - `CalendarSource` trait for pluggable backends
- `calendars/local_calendar.rs` - Local calendar implementation
- `calendars/caldav_calendar.rs` - CalDAV calendar (WIP)

### Dialog Management (`dialogs/`)
- `dialogs/mod.rs` - `ActiveDialog` enum for all dialog types
- `dialogs/manager.rs` - `DialogManager` for dialog lifecycle

### Database Layer (`database/`)
- `database/schema.rs` - SQLite schema and queries for calendars and events

### Logging (`logging.rs`)
Centralized logging configuration for the application. Use `log` macros throughout the codebase:
```rust
use log::{debug, info, warn, error, trace};
info!("Operation completed");
debug!("Processing uid={}", uid);
```

Control log level via `RUST_LOG` environment variable:
- `RUST_LOG=debug cargo run` - Enable debug logs
- `RUST_LOG=sol_calendar::services=debug` - Debug for services only

### Services Layer (`services/`)
Service handlers centralize business logic and act as middleware between the UI/update layer and the protocol/storage layer.

- `services/event_handler.rs` - Event CRUD operations
  - Routes events to correct protocol (local vs remote)
  - Validates events before saving
  - Handles sync and conflict resolution
  - Centralizes cache invalidation

- `services/calendar_handler.rs` - Calendar management
  - Create, update, delete calendars
  - Toggle visibility, change colors
  - Generate unique calendar IDs
  - Validate calendar data

- `services/settings_handler.rs` - Application settings
  - Load/save settings from disk
  - Toggle week numbers display
  - Reset to defaults

- `services/sync_handler.rs` - Synchronization
  - Sync individual or all calendars
  - Track sync status and errors
  - Detect remote calendar requirements

- `services/export_handler.rs` - Import/Export
  - Export events to iCalendar (.ics) format
  - Export single events or entire calendars
  - Read iCalendar files (import WIP)

### Constants
- `ui_constants.rs` - UI dimensions, spacing, and color values (consolidated)

## Internationalization

Translations use Mozilla Fluent format in `i18n/{locale}/sol_calendar.ftl`. Supported locales: cs, da, de, el, en, es, fi, fr, it, nl, no, pl, pt, ro, sv, uk.

Use the `fl!()` macro to get localized strings:
```rust
fl!("app-title")  // Returns localized string
```

## Key Patterns

- All views are pure functions returning `Element<Message>`
- State changes only through message passing via `update.rs`
- `CalendarCache` should be used for expensive date calculations
- Keyboard shortcuts defined in `keyboard.rs` using `menu::KeyBind`
- Calendar backends implement `CalendarSource` trait
- **Event operations go through EventHandler**, not directly to calendars
- **Protocols are storage-agnostic** - they only know how to read/write events
- **EventHandler is the single point** for event CRUD operations

## Security Best Practices

### ⚠️ Avoid Logging Sensitive Information

**NEVER log user-generated content or personally identifiable information.** This includes:

❌ **DO NOT LOG:**
- Event summaries/titles (may contain sensitive appointments)
- Event notes/descriptions
- Event locations
- Invitee email addresses
- User credentials (passwords, tokens, API keys)
- Full event objects

✅ **SAFE TO LOG:**
- Event UIDs (non-sensitive identifiers)
- Calendar IDs
- Operation types (create, update, delete)
- Error codes and status messages
- Technical metadata (timestamps, counts)

**Example:**
```rust
// ❌ BAD - Logs sensitive event title
info!("Creating event '{}'", event.summary);

// ✅ GOOD - Logs only UID
info!("Creating event uid={}", event.uid);

// ❌ BAD - Logs user email
debug!("User {} logged in", email);

// ✅ GOOD - Logs only action
debug!("User authentication successful");
```

### 🔒 Avoid Cleartext Transmission of Sensitive Information

**ALWAYS use HTTPS for network communication.** Never transmit credentials or calendar data over unencrypted HTTP connections.

**Requirements:**

1. **Enforce HTTPS at Client Level**
   - Use `reqwest::Client::builder().https_only(true)` for all HTTP clients
   - Reject HTTP URLs with clear error messages

2. **Validate URLs Before Use**
   - Check that all user-supplied URLs start with `https://`
   - Reject or upgrade HTTP URLs to HTTPS
   - Validate at multiple layers (client, protocol, calendar)

3. **Defense in Depth**
   - Validate HTTPS at the earliest point possible
   - Re-validate at each layer of abstraction
   - Never assume input has been validated upstream

**Example:**
```rust
// ✅ GOOD - Validates HTTPS before creating client
pub fn new(server_url: String, username: String, password: String) -> Result<Self, Box<dyn Error>> {
    // Security: Enforce HTTPS-only connections
    if !server_url.starts_with("https://") {
        return Err(format!(
            "Server URL must use HTTPS for secure transmission. Got: {}",
            server_url
        ).into());
    }

    // Configure client to enforce HTTPS
    let client = Client::builder()
        .https_only(true)
        .build()?;

    Ok(Self { server_url, username, password, client })
}

// ❌ BAD - Accepts any URL without validation
pub fn new(server_url: String, username: String, password: String) -> Self {
    Self {
        server_url,  // Could be http://
        client: Client::new(),  // Allows HTTP
    }
}
```

**Security Checklist for Network Code:**
- [ ] All `CalDavClient` instances enforce HTTPS
- [ ] User-supplied URLs validated before use
- [ ] Error messages guide users to HTTPS
- [ ] No hardcoded HTTP URLs in codebase
- [ ] Credentials never logged or transmitted over HTTP

## Data Flow

### Event Creation Flow (UI → Database → UI)

```
1. User Action (click date, drag selection, or event dialog)
   ↓
2. Message Generated (SelectionEnd, CommitQuickEvent, ConfirmEventDialog)
   ↓
3. Update Handler (update/event.rs)
   - Creates CalendarEvent struct with uid, summary, dates, times
   ↓
4. EventHandler::add_event() (services/event_handler.rs)
   - Validates event (title, time range)
   - Finds target calendar
   - Routes through CalendarSource trait
   ↓
5. LocalCalendar::add_event() (calendars/local_calendar.rs)
   - Calls Database::insert_event()
   - Updates cached_events vec
   ↓
6. Database::insert_event() (database/schema.rs)
   - Serializes complex types (RepeatFrequency, AlertTime)
   - Stores in SQLite events table
   ↓
7. app.refresh_cached_events()
   - Rebuilds cached_month_events/cached_week_events HashMaps
   ↓
8. Views re-render with new DisplayEvent objects
```

### State Synchronization

When `selected_date` changes, all views sync:
```rust
pub fn sync_views_to_selected_date(&mut self) {
    self.cache.set_current(date.year(), date.month());
    self.week_state = WeekState::new(date, ...);
    self.day_state = DayState::new(date, ...);
    self.year_state = YearState::new(date.year());
    self.mini_calendar_state = CalendarState::new(date.year(), date.month());
    self.refresh_cached_events();
}
```

## Event Model (caldav.rs)

```rust
pub struct CalendarEvent {
    pub uid: String,                    // UUID4
    pub summary: String,                // Title (required)
    pub location: Option<String>,
    pub all_day: bool,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub travel_time: TravelTime,
    pub repeat: RepeatFrequency,        // Never, Daily, Weekly, Monthly, Yearly, Custom(RRULE)
    pub invitees: Vec<String>,
    pub alert: AlertTime,
    pub alert_second: Option<AlertTime>,
    pub attachments: Vec<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
}
```

**RepeatFrequency**: `Never | Daily | Weekly | Biweekly | Monthly | Yearly | Custom(String)`

**AlertTime**: `None | AtTime | FiveMinutes | TenMinutes | FifteenMinutes | ThirtyMinutes | OneHour | TwoHours | OneDay | TwoDays | OneWeek | Custom(i32)`

**DisplayEvent** (rendered version with calendar color):
```rust
pub struct DisplayEvent {
    pub uid: String,
    pub summary: String,
    pub color: String,                 // Hex from calendar
    pub all_day: bool,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub span_start: Option<NaiveDate>, // Multi-day start
    pub span_end: Option<NaiveDate>,
}
```

## Database Schema (SQLite)

**Location**: `~/.local/share/sol-calendar/sol.db`

```sql
CREATE TABLE events (
    uid TEXT PRIMARY KEY,
    calendar_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    location TEXT,
    all_day INTEGER NOT NULL DEFAULT 0,
    start_time TEXT NOT NULL,           -- RFC3339 UTC
    end_time TEXT NOT NULL,
    travel_time TEXT NOT NULL DEFAULT 'None',    -- JSON
    repeat TEXT NOT NULL DEFAULT 'Never',        -- JSON
    invitees TEXT NOT NULL DEFAULT '[]',         -- JSON array
    alert TEXT NOT NULL DEFAULT 'None',          -- JSON
    alert_second TEXT,
    attachments TEXT NOT NULL DEFAULT '[]',
    url TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Note**: Calendar metadata (name, color, enabled) stored in JSON config, not database.

## Key Abstractions

### CalendarSource Trait (calendars/calendar_source.rs)
```rust
pub trait CalendarSource: Debug + Send {
    fn info(&self) -> &CalendarInfo;
    fn is_enabled(&self) -> bool;
    fn fetch_events(&self) -> Result<Vec<CalendarEvent>>;
    fn add_event(&mut self, event: CalendarEvent) -> Result<()>;
    fn update_event(&mut self, event: CalendarEvent) -> Result<()>;
    fn delete_event(&mut self, uid: &str) -> Result<()>;
    fn sync(&mut self) -> Result<()>;
}
```

### CalendarManager (calendars/mod.rs)
- Manages multiple CalendarSource implementations
- Routes operations to correct calendar
- Provides `get_display_events_for_month()` / `get_display_events_for_week()`

### ActiveDialog Enum (dialogs/mod.rs)
Only one dialog open at a time:
```rust
pub enum ActiveDialog {
    None,
    QuickEvent { date, text, start_time, end_time },
    EventCreate,
    EventEdit { uid },
    CalendarCreate { name, color },
    CalendarEdit { id, name, color },
    EventDelete { uid, is_recurring, ... },
    ConfirmDeleteCalendar { id },
    ColorPicker { calendar_id },
}
```

## File Quick Reference

| Task | Files to Modify |
|------|-----------------|
| New event property | `caldav.rs`, `database/schema.rs`, `update/event.rs`, components |
| New message type | `message.rs`, `update/mod.rs`, handler function |
| New keyboard shortcut | `keyboard.rs`, `menu_action.rs`, `message.rs`, `update/mod.rs` |
| New calendar type | Implement `CalendarSource` trait in `calendars/` |
| Improve repeating events | `caldav.rs` (RepeatFrequency), expansion logic in views |
| Add new view | `views/` module, `models/` state, `CalendarView` enum |

## Development Flags (Debug Only)

```bash
cargo run -- --dev-reset-db      # Clear all events
cargo run -- --dev-seed-data     # Generate demo events for a year
```

## Keyboard Shortcuts

Defined in `keyboard.rs`:
- `Ctrl+Shift+N` - New event
- `Ctrl+Shift+T` - Today
- `Ctrl+Shift+D/W/M/Y` - Day/Week/Month/Year view
- `Ctrl+Shift+Arrow` - Navigate period
- `Ctrl+Shift+[/]` - Cycle views
- `Delete` - Delete selected event
