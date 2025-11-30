<div align="center">
  <img src="res/icons/hicolor/scalable/apps/io.github.xarbit.SolCalendar.svg" alt="Sol Calendar Icon" width="128" height="128">

  # Sol - a Calendar for the COSMIC Desktop
</div>

![Month View](images/image1.png)

![Week view](images/week.png)

![Day view](images/day.png)

![Year view](images/year.png)

> ⚠️ **WORK IN PROGRESS** - This project is in early development and is **not ready for production use**. Many features are incomplete or missing. Use at your own risk!

A modern calendar application built with [libcosmic](https://github.com/pop-os/libcosmic), featuring CalDAV support for seamless calendar synchronization.

## About

Sol is a native calendar application designed for the COSMIC desktop environment. Built using libcosmic's widget system, it provides a clean, intuitive interface inspired by Apple Calendar's design while following COSMIC's design language and responsive layout patterns.

The application will support CalDAV protocol for synchronizing events with calendar servers like Nextcloud, Radicale, and other standard CalDAV-compatible services.

## Current Status

This project is in **active development**. The UI foundation is in place, but core calendar functionality is still being implemented.

### ✅ Implemented Features

#### Core UI
- Apple Calendar-inspired layout with responsive left sidebar
- Mini calendar in sidebar for quick date navigation
- Day selection with visual feedback (outlined today, filled selection)
- Square day cells with 4px rounded corners (theme-independent)
- Instant responsive UI that adapts to window size
- Sidebar overlay mode for small screens (COSMIC Files-style)
- COSMIC-style menu bar (File, Edit, View)
- Navigation controls (Previous/Next/Today buttons)

#### Multiple View Modes
- **Month View**: Full month calendar grid with week numbers (optional)
  - Quick event creation by clicking on day cells
  - Multi-day event selection via drag
  - Event chips with color coding
- **Week View**: Week schedule with hourly time slots
  - Side-by-side layout for overlapping events
  - Current time indicator (red line spanning all days, dot on today)
  - Auto-scroll to current time when entering view
  - Time-slot drag selection for creating timed events
  - All-day events section at the top
  - Drag-and-drop event rescheduling
- **Day View**: Single day detailed schedule with hourly breakdown
- **Year View**: 12-month overview in 3×4 grid (Apple Calendar style)

#### Event Management
- Quick event creation via click or keyboard
- Timed event creation with drag selection in week view
- Event editing dialog with full details
- Drag-and-drop event rescheduling (month and week views)
- Event deletion
- SQLite database persistence

#### Navigation & Controls
- View switcher buttons (Day/Week/Month/Year) in toolbar
- Keyboard shortcuts for quick navigation:
  - `Ctrl+1` - Month View
  - `Ctrl+2` - Week View
  - `Ctrl+3` - Day View
  - `Ctrl+4` - Year View
  - `Ctrl+N` - New Event
  - `T` - Jump to Today
  - `Left/Right` - Navigate previous/next period

#### Localization
- System locale detection with fallback to English
- Localized month and day names
- First day of week respects locale (Monday/Sunday)
- Week number calculation (ISO 8601)
- 16 languages supported (cs, da, de, el, en, es, fi, fr, it, nl, no, pl, pt, ro, sv, uk)

#### Calendar Management
- Multiple calendar support with color coding
- Calendar visibility toggle
- Custom color picker for calendars
- Create, edit, and delete calendars
- Default calendars: Personal (blue), Work (purple)

### 🚧 Work In Progress

- [ ] CalDAV server configuration UI
- [ ] Active CalDAV synchronization
- [ ] Event notifications/alerts
- [ ] Recurring events
- [ ] Background sync
- [ ] Search functionality
- [ ] Event invitees

## Building

### Prerequisites

- Rust (latest stable version)
- libcosmic dependencies (automatically fetched from git)

### Compile

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

## Architecture

Sol follows the **Elm/MVU (Model-View-Update)** architecture pattern, which is standard for libcosmic applications:

### Module Organization

```
src/
├── app.rs                  # Main application state and COSMIC framework integration
├── main.rs                 # Entry point
├── message.rs              # Application message enum
├── keyboard.rs             # Centralized keyboard shortcuts
├── layout.rs               # Responsive layout management
├── selection.rs            # Drag selection and event drag state
│
├── update/                 # Message handling (split by domain)
│   ├── mod.rs              # Main message dispatcher
│   ├── navigation.rs       # View navigation handlers
│   ├── calendar.rs         # Calendar management handlers
│   ├── event.rs            # Event CRUD handlers
│   └── selection/          # Selection and drag handlers
│
├── models/                 # Domain models and state
│   ├── calendar_state.rs   # Month calendar state with caching
│   ├── week_state.rs       # Week view state
│   ├── day_state.rs        # Day view state
│   └── year_state.rs       # Year view state
│
├── views/                  # View rendering functions (pure functions)
│   ├── main_view.rs        # Main content coordinator
│   ├── month.rs            # Month grid view
│   ├── week.rs             # Week schedule view with time indicator
│   ├── day.rs              # Day schedule view
│   ├── year.rs             # Year overview
│   └── sidebar.rs          # Sidebar layout
│
├── components/             # Reusable UI components
│   ├── day_cell.rs         # Individual day cell with events
│   ├── mini_calendar.rs    # Mini calendar widget
│   ├── calendar_list.rs    # Calendar list widget
│   ├── color_picker.rs     # Color selection widget
│   ├── toolbar.rs          # Navigation toolbar
│   ├── time_grid.rs        # Hour-based time grid for week/day views
│   ├── event_chip.rs       # Event display chips
│   └── header_menu.rs      # Application menu bar
│
├── dialogs/                # Dialog management
│   ├── mod.rs              # Dialog types and state
│   └── manager.rs          # Dialog lifecycle management
│
├── services/               # Business logic services
│   ├── calendar_handler.rs # Calendar CRUD operations
│   ├── event_handler.rs    # Event CRUD operations
│   └── settings_handler.rs # Settings persistence
│
├── database/               # Data persistence
│   └── schema.rs           # SQLite schema and queries
│
├── calendars/              # Calendar data sources
│   ├── calendar_source.rs  # Calendar trait definition
│   ├── local_calendar.rs   # Local calendar implementation
│   └── caldav_calendar.rs  # CalDAV calendar implementation (WIP)
│
├── locale.rs               # Locale detection and formatting
├── localized_names.rs      # Localized month/day names
├── cache.rs                # Calendar state caching
├── caldav.rs               # CalDAV protocol types
├── settings.rs             # Persistent app settings
├── ui_constants.rs         # UI dimensions, spacing, and colors
└── styles.rs               # Custom styles for containers
```

### Key Architecture Patterns

- **MVU Pattern**: Clean separation of Model (state), View (rendering), and Update (state transitions)
- **Pure View Functions**: All views are pure functions that take state and return UI elements
- **Centralized State**: Single source of truth in `CosmicCalendar` struct
- **Message-Based Updates**: All state changes happen through message passing
- **Caching Layer**: `CalendarCache` pre-computes calendar states for performance
- **Calendar Abstraction**: `CalendarSource` trait enables pluggable calendar backends

## Technology Stack

- **[libcosmic](https://github.com/pop-os/libcosmic)**: Modern UI framework for COSMIC desktop built on iced
- **chrono**: Date and time handling with timezone support
- **chrono-tz**: Timezone database
- **i18n-embed**: Internationalization framework
- **fluent**: Localization system (Mozilla Fluent)
- **icalendar**: iCalendar format parsing and generation
- **reqwest**: HTTP client for CalDAV operations
- **serde**: Serialization/deserialization
- **dirs**: Platform-specific directory handling
- **ron**: Rusty Object Notation for settings storage

## Planned CalDAV Support

The application will support full CalDAV protocol integration:

- Connect to any CalDAV-compatible server (Nextcloud, Radicale, etc.)
- Synchronize events bidirectionally
- Local event caching for offline access
- Support for multiple calendar accounts

### Planned CalDAV Configuration

Users will be able to configure:
1. CalDAV server URL
2. Username and password/app-specific password
3. Which calendars to sync
4. Sync interval

## Design Philosophy

- **Native COSMIC integration**: Uses libcosmic for native look and feel
- **Responsive design**: Adapts to different window sizes with instant transitions
- **Theme-independent styling**: Critical UI elements maintain consistent appearance
- **Offline-first**: Local storage with server sync
- **Privacy-focused**: Events stored locally by default

## Recent Improvements

### Week View Polish (Latest)
- **Current time indicator**: Red line spanning all days with dot on today's column
- **Auto-scroll**: Week view automatically scrolls to current time on entry
- **Timer updates**: Time indicator updates every 30 seconds
- **Overlapping events**: Side-by-side layout for concurrent events
- **Time-slot selection**: Drag to select time range for new timed events
- **Event drag-and-drop**: Reschedule events by dragging to new dates

### Event Management
- **Quick event creation**: Click day cell or use keyboard shortcut
- **Event dialog**: Full-featured editor with title, location, times, calendar selection
- **Database persistence**: SQLite storage with automatic schema creation
- **Event chips**: Color-coded display in month and week views

### Architecture Improvements
- **Modular update handlers**: Split `update.rs` into domain-specific modules
- **Dialog management**: Centralized dialog state in `dialogs/manager.rs`
- **Service layer**: Business logic separated into `services/` directory
- **Selection state**: Unified drag selection for dates and time slots

## Contributing

Contributions are welcome! However, please note that this project is in early development and the architecture may change significantly. Feel free to:

- Report bugs and issues
- Suggest features
- Submit pull requests
- Improve documentation

## License

This project is licensed under the GNU General Public License v3.0 (GPLv3). See the [LICENSE](LICENSE) file for details.

## Disclaimer

**This software is NOT ready for production use.** Features are incomplete, bugs are expected, and data loss may occur. Do not rely on this application for important calendar events at this time.
