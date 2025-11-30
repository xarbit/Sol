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
- `update.rs` - Message handling and state mutations
- `keyboard.rs` - Centralized keyboard shortcuts (single source of truth)

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
- `calendars/caldav_calendar.rs` - CalDAV calendar (uses CalDavProtocol)

### Protocol Layer (`protocols/`)
Protocol implementations handle the actual storage/sync operations. The EventHandler middleware routes events to the appropriate protocol.

```
UI → Update Handlers → EventHandler (middleware) → Protocols
                                                      ├── LocalProtocol (SQLite)
                                                      └── CalDavProtocol (HTTP/CalDAV)
```

- `protocols/mod.rs` - `Protocol` trait definition
- `protocols/local.rs` - Local SQLite storage protocol
- `protocols/caldav.rs` - CalDAV HTTP protocol

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
- `layout_constants.rs` - UI dimensions and spacing
- `color_constants.rs` - Color values

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
