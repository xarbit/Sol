mod caldav;
mod storage;

use chrono::Datelike;
use cosmic::app::{Core, Settings};
use cosmic::iced::{alignment, Background, Border, Color, Length, Shadow, Vector};
use cosmic::iced::widget::stack;
use cosmic::widget::{self, button, column, container, divider, layer_container, mouse_area, row, scrollable};
use cosmic::{Application, Element};
use storage::LocalStorage;

const APP_ID: &str = "io.github.xarbit.SolCalendar";

pub fn main() -> cosmic::iced::Result {
    cosmic::app::run::<CosmicCalendar>(Settings::default(), ())
}

struct CosmicCalendar {
    core: Core,
    current_view: CalendarView,
    current_year: i32,
    current_month: u32,
    selected_day: Option<u32>,
    storage: LocalStorage,
    show_sidebar: bool,
    show_search: bool,
}

impl Default for CosmicCalendar {
    fn default() -> Self {
        let now = chrono::Local::now();
        let storage_path = LocalStorage::get_storage_path();
        let storage = LocalStorage::load_from_file(&storage_path).unwrap_or_default();

        CosmicCalendar {
            core: Core::default(),
            current_view: CalendarView::Month,
            current_year: now.year(),
            current_month: now.month(),
            selected_day: Some(now.day()),
            storage,
            show_sidebar: true,
            show_search: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalendarView {
    Month,
    Week,
    Day,
}

impl Default for CalendarView {
    fn default() -> Self {
        CalendarView::Month
    }
}

#[derive(Debug, Clone)]
enum Message {
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(u32),
    ToggleSidebar,
    ToggleSearch,
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,
    NewEvent,
    Settings,
    About,
}


impl Application for CosmicCalendar {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let now = chrono::Local::now();
        let storage_path = LocalStorage::get_storage_path();
        let storage = LocalStorage::load_from_file(&storage_path).unwrap_or_default();

        let app = CosmicCalendar {
            core,
            current_view: CalendarView::Month,
            current_year: now.year(),
            current_month: now.month(),
            selected_day: Some(now.day()),
            storage,
            show_sidebar: true,
            show_search: false,
        };
        (app, cosmic::app::Task::none())
    }


    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![
            button::icon(widget::icon::from_name("sidebar-show-symbolic"))
                .on_press(Message::ToggleSidebar)
                .into(),
            widget::button::text("File")
                .on_press(Message::NewEvent)
                .padding([4, 12])
                .into(),
            widget::button::text("Edit")
                .on_press(Message::Settings)
                .padding([4, 12])
                .into(),
            widget::button::text("View")
                .on_press(Message::ChangeView(CalendarView::Month))
                .padding([4, 12])
                .into(),
            widget::button::text("Help")
                .on_press(Message::About)
                .padding([4, 12])
                .into(),
        ]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        vec![
            button::icon(widget::icon::from_name("system-search-symbolic"))
                .on_press(Message::ToggleSearch)
                .into()
        ]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Apple Calendar layout: left sidebar + main content
        let is_condensed = self.core.is_condensed();

        // Build base layout with sidebar inline when appropriate
        let base_content = if !is_condensed && self.show_sidebar {
            // Large screen: sidebar inline on left
            row()
                .spacing(0)
                .push(self.render_sidebar())
                .push(divider::vertical::default())
                .push(self.render_main_content())
                .into()
        } else if !is_condensed {
            // Large screen, sidebar hidden
            self.render_main_content()
        } else {
            // Condensed screen: just main content as base
            self.render_main_content()
        };

        // In condensed mode with sidebar toggled on, show it as overlay
        if is_condensed && self.show_sidebar {
            let overlay_sidebar = container(
                container(self.render_sidebar())
                    .style(|theme: &cosmic::Theme| {
                        container::Style {
                            background: Some(Background::Color(theme.cosmic().background.base.into())),
                            border: Border {
                                width: 0.0,
                                ..Default::default()
                            },
                            shadow: Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                                offset: Vector::new(2.0, 0.0),
                                blur_radius: 10.0,
                            },
                            ..Default::default()
                        }
                    })
            )
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Left);

            stack![base_content, overlay_sidebar].into()
        } else {
            base_content
        }
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::ChangeView(view) => {
                self.current_view = view;
            }
            Message::PreviousPeriod => {
                match self.current_view {
                    CalendarView::Month => {
                        if self.current_month == 1 {
                            self.current_month = 12;
                            self.current_year -= 1;
                        } else {
                            self.current_month -= 1;
                        }
                    }
                    CalendarView::Week => {
                        // Week navigation logic
                    }
                    CalendarView::Day => {
                        // Day navigation logic
                    }
                }
            }
            Message::NextPeriod => {
                match self.current_view {
                    CalendarView::Month => {
                        if self.current_month == 12 {
                            self.current_month = 1;
                            self.current_year += 1;
                        } else {
                            self.current_month += 1;
                        }
                    }
                    CalendarView::Week => {
                        // Week navigation logic
                    }
                    CalendarView::Day => {
                        // Day navigation logic
                    }
                }
            }
            Message::Today => {
                let now = chrono::Local::now();
                self.current_year = now.year();
                self.current_month = now.month();
                self.selected_day = Some(now.day());
            }
            Message::SelectDay(day) => {
                self.selected_day = Some(day);
            }
            Message::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
            }
            Message::ToggleSearch => {
                self.show_search = !self.show_search;
            }
            Message::MiniCalendarPrevMonth => {
                if self.current_month == 1 {
                    self.current_month = 12;
                    self.current_year -= 1;
                } else {
                    self.current_month -= 1;
                }
            }
            Message::MiniCalendarNextMonth => {
                if self.current_month == 12 {
                    self.current_month = 1;
                    self.current_year += 1;
                } else {
                    self.current_month += 1;
                }
            }
            Message::NewEvent => {
                // TODO: Open new event dialog
                println!("New Event requested");
            }
            Message::Settings => {
                // TODO: Open settings dialog
                println!("Settings requested");
            }
            Message::About => {
                // TODO: Show about dialog
                println!("About requested");
            }
        }
        cosmic::app::Task::none()
    }
}

impl CosmicCalendar {
    fn render_sidebar(&self) -> Element<'_, Message> {
        let mini_calendar = self.render_mini_calendar();

        let calendars_section = column()
            .spacing(8)
            .padding(12)
            .push(widget::text::body("Calendars").size(14))
            .push(
                row()
                    .spacing(8)
                    .push(widget::checkbox("", true))
                    .push(widget::text("Personal"))
            )
            .push(
                row()
                    .spacing(8)
                    .push(widget::checkbox("", true))
                    .push(widget::text("Work"))
            );

        let sidebar_content = column()
            .spacing(20)
            .padding(16)
            .push(mini_calendar)
            .push(divider::horizontal::default())
            .push(calendars_section);

        container(scrollable(sidebar_content))
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .into()
    }

    fn render_mini_calendar(&self) -> Element<'_, Message> {
        let date = chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
        let month_year = format!("{}", date.format("%B %Y"));

        let header = row()
            .spacing(8)
            .push(
                button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::MiniCalendarPrevMonth)
                    .padding(4)
            )
            .push(
                container(widget::text::body(month_year).size(14))
                    .width(Length::Fill)
            )
            .push(
                button::icon(widget::icon::from_name("go-next-symbolic"))
                    .on_press(Message::MiniCalendarNextMonth)
                    .padding(4)
            );

        let first_day = chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let days_in_month = if self.current_month == 12 {
            chrono::NaiveDate::from_ymd_opt(self.current_year + 1, 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        } else {
            chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month + 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        };

        let mut grid = column().spacing(4);

        // Weekday headers (abbreviated)
        let header_row = row()
            .spacing(2)
            .push(widget::text("M").width(Length::Fill).size(11))
            .push(widget::text("T").width(Length::Fill).size(11))
            .push(widget::text("W").width(Length::Fill).size(11))
            .push(widget::text("T").width(Length::Fill).size(11))
            .push(widget::text("F").width(Length::Fill).size(11))
            .push(widget::text("S").width(Length::Fill).size(11))
            .push(widget::text("S").width(Length::Fill).size(11));

        grid = grid.push(header_row);

        // Calendar days
        let mut weeks = vec![];
        let mut current_week = vec![];

        for _ in 0..first_weekday {
            current_week.push(None);
        }

        for day in 1..=days_in_month {
            current_week.push(Some(day as u32));
            if current_week.len() == 7 {
                weeks.push(current_week.clone());
                current_week.clear();
            }
        }

        if !current_week.is_empty() {
            while current_week.len() < 7 {
                current_week.push(None);
            }
            weeks.push(current_week);
        }

        let today = chrono::Local::now();
        let is_current_month = today.year() == self.current_year && today.month() == self.current_month;

        for week in weeks {
            let mut week_row = row().spacing(2);
            for day_opt in week {
                if let Some(day) = day_opt {
                    let is_today = is_current_month && today.day() == day;
                    let is_selected = self.selected_day == Some(day);

                    let day_button = if is_today {
                        widget::button::suggested(day.to_string())
                            .on_press(Message::SelectDay(day))
                            .padding(4)
                            .width(Length::Fixed(32.0))
                    } else if is_selected {
                        widget::button::standard(day.to_string())
                            .on_press(Message::SelectDay(day))
                            .padding(4)
                            .width(Length::Fixed(32.0))
                    } else {
                        widget::button::text(day.to_string())
                            .on_press(Message::SelectDay(day))
                            .padding(4)
                            .width(Length::Fixed(32.0))
                    };
                    week_row = week_row.push(day_button);
                } else {
                    week_row = week_row.push(container(widget::text("")).width(Length::Fixed(32.0)));
                }
            }
            grid = grid.push(week_row);
        }

        column()
            .spacing(12)
            .push(header)
            .push(grid)
            .into()
    }

    fn render_main_content(&self) -> Element<'_, Message> {
        // Toolbar
        let date = chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
        let period_text = match self.current_view {
            CalendarView::Month => format!("{}", date.format("%B %Y")),
            CalendarView::Week => format!("Week of {}", date.format("%B %d, %Y")),
            CalendarView::Day => format!("{}", date.format("%B %d, %Y")),
        };

        let toolbar_left = row()
            .spacing(8)
            .push(widget::button::standard("Today").on_press(Message::Today))
            .push(
                button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::PreviousPeriod)
                    .padding(8)
            )
            .push(
                button::icon(widget::icon::from_name("go-next-symbolic"))
                    .on_press(Message::NextPeriod)
                    .padding(8)
            )
            .push(widget::text::title4(period_text));

        let view_switcher = row()
            .spacing(4)
            .push(
                if self.current_view == CalendarView::Day {
                    widget::button::suggested("Day").on_press(Message::ChangeView(CalendarView::Day))
                } else {
                    widget::button::standard("Day").on_press(Message::ChangeView(CalendarView::Day))
                }
            )
            .push(
                if self.current_view == CalendarView::Week {
                    widget::button::suggested("Week").on_press(Message::ChangeView(CalendarView::Week))
                } else {
                    widget::button::standard("Week").on_press(Message::ChangeView(CalendarView::Week))
                }
            )
            .push(
                if self.current_view == CalendarView::Month {
                    widget::button::suggested("Month").on_press(Message::ChangeView(CalendarView::Month))
                } else {
                    widget::button::standard("Month").on_press(Message::ChangeView(CalendarView::Month))
                }
            );

        let toolbar = row()
            .padding(16)
            .push(toolbar_left)
            .push(container(widget::text("")).width(Length::Fill))
            .push(view_switcher);

        let calendar_view = match self.current_view {
            CalendarView::Month => self.render_month_view(),
            CalendarView::Week => self.render_week_view(),
            CalendarView::Day => self.render_day_view(),
        };

        column()
            .spacing(0)
            .push(toolbar)
            .push(divider::horizontal::default())
            .push(calendar_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_month_view(&self) -> Element<'_, Message> {
        let first_day = chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let days_in_month = if self.current_month == 12 {
            chrono::NaiveDate::from_ymd_opt(self.current_year + 1, 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        } else {
            chrono::NaiveDate::from_ymd_opt(self.current_year, self.current_month + 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        };

        let mut grid = column().spacing(1).padding(20);

        // Weekday headers
        let header_row = row()
            .spacing(1)
            .push(container(widget::text("Monday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Tuesday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Wednesday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Thursday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Friday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Saturday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill))
            .push(container(widget::text("Sunday").size(12)).width(Length::Fill).padding(8).center_x(Length::Fill));

        grid = grid.push(header_row);

        // Calendar days
        let mut weeks = vec![];
        let mut current_week = vec![];

        for _ in 0..first_weekday {
            current_week.push(None);
        }

        for day in 1..=days_in_month {
            current_week.push(Some(day as u32));
            if current_week.len() == 7 {
                weeks.push(current_week.clone());
                current_week.clear();
            }
        }

        if !current_week.is_empty() {
            while current_week.len() < 7 {
                current_week.push(None);
            }
            weeks.push(current_week);
        }

        let today = chrono::Local::now();
        let is_current_month = today.year() == self.current_year && today.month() == self.current_month;

        // Render weeks with cells
        for week in weeks {
            let mut week_row = row().spacing(1).height(Length::Fill);
            for day_opt in week {
                let cell = if let Some(day) = day_opt {
                    let is_today = is_current_month && today.day() == day;
                    let is_selected = self.selected_day == Some(day);

                    // Create day cell with explicit 4px radius - use mouse_area instead of button
                    let day_cell = if is_today {
                        // Today: outlined with accent color border (not filled)
                        container(
                            container(widget::text::title4(day.to_string()))
                                .padding([4, 8, 0, 0])  // Top-right padding
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Right)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(4)
                        .style(|theme: &cosmic::Theme| {
                            container::Style {
                                background: None,
                                border: Border {
                                    color: theme.cosmic().accent_color().into(),
                                    width: 2.0,
                                    radius: [4.0, 4.0, 4.0, 4.0].into(),  // Force 4px radius
                                },
                                ..Default::default()
                            }
                        })
                    } else if is_selected {
                        // Selected: filled with accent color
                        container(
                            container(widget::text::title4(day.to_string()))
                                .padding([4, 8, 0, 0])  // Top-right padding
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Right)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(4)
                        .style(|theme: &cosmic::Theme| {
                            container::Style {
                                background: Some(Background::Color(theme.cosmic().accent_color().into())),
                                border: Border {
                                    radius: [4.0, 4.0, 4.0, 4.0].into(),  // Force 4px radius
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
                    } else {
                        // Normal day - light border
                        container(
                            container(widget::text(day.to_string()))
                                .padding([4, 8, 0, 0])  // Top-right padding
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Right)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(4)
                        .style(|theme: &cosmic::Theme| {
                            container::Style {
                                background: None,
                                border: Border {
                                    color: Color::from_rgba(0.5, 0.5, 0.5, 0.2).into(),  // Light gray border
                                    width: 1.0,
                                    radius: [4.0, 4.0, 4.0, 4.0].into(),  // Force 4px radius
                                },
                                ..Default::default()
                            }
                        })
                    };

                    // Wrap in mouse_area for click handling - no theme button styling
                    mouse_area(day_cell)
                        .on_press(Message::SelectDay(day))
                } else {
                    mouse_area(container(widget::text("")).padding(8))
                };

                week_row = week_row.push(
                    container(cell)
                        .width(Length::Fill)
                        .height(Length::Fill)
                );
            }
            grid = grid.push(week_row);
        }

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_week_view(&self) -> Element<'_, Message> {
        let content = column()
            .spacing(20)
            .padding(40)
            .push(widget::text::title2("Week View"))
            .push(widget::text("Week view coming soon..."));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn render_day_view(&self) -> Element<'_, Message> {
        let content = column()
            .spacing(20)
            .padding(40)
            .push(widget::text::title2("Day View"))
            .push(widget::text("Day view coming soon..."));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
