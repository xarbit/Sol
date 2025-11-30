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
