mod cache;
mod caldav;
mod components;
mod message;
mod models;
mod storage;
mod views;

use cache::CalendarCache;
use chrono::Datelike;
use cosmic::app::{Core, Settings};
use cosmic::iced::{alignment, Background, Border, Color, Length, Shadow, Vector};
use cosmic::iced::widget::stack;
use cosmic::widget::{self, button, column, container, divider, row};
use cosmic::{Application, Element};
use message::Message;
use storage::LocalStorage;
use views::CalendarView;

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
    #[allow(dead_code)] // Will be used for event storage in future
    storage: LocalStorage,
    show_sidebar: bool,
    show_search: bool,
    // Centralized cache for calendar states and formatted text
    cache: CalendarCache,
}

impl Default for CosmicCalendar {
    fn default() -> Self {
        let now = chrono::Local::now();
        let storage_path = LocalStorage::get_storage_path();
        let storage = LocalStorage::load_from_file(&storage_path).unwrap_or_default();

        let year = now.year();
        let month = now.month();

        // Create cache with current month
        let cache = CalendarCache::new(year, month);

        CosmicCalendar {
            core: Core::default(),
            current_view: CalendarView::Month,
            current_year: year,
            current_month: month,
            selected_day: Some(now.day()),
            storage,
            show_sidebar: true,
            show_search: false,
            cache,
        }
    }
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

        let year = now.year();
        let month = now.month();

        // Create cache and pre-cache surrounding months
        let mut cache = CalendarCache::new(year, month);
        cache.precache_surrounding(1, 2); // Pre-cache 1 month before, 2 months after

        let app = CosmicCalendar {
            core,
            current_view: CalendarView::Month,
            current_year: year,
            current_month: month,
            selected_day: Some(now.day()),
            storage,
            show_sidebar: true,
            show_search: false,
            cache,
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
                        // Update cache and pre-cache surrounding months
                        self.cache.set_current(self.current_year, self.current_month);
                        self.cache.precache_surrounding(1, 2);
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
                        // Update cache and pre-cache surrounding months
                        self.cache.set_current(self.current_year, self.current_month);
                        self.cache.precache_surrounding(1, 2);
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
                // Update cache and pre-cache surrounding months
                self.cache.set_current(self.current_year, self.current_month);
                self.cache.precache_surrounding(1, 2);
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
        // Use calendar cache module
        views::render_sidebar(self.cache.current_state(), self.selected_day)
    }

    fn render_main_content(&self) -> Element<'_, Message> {
        // Toolbar - use cached period text from cache module
        let period_text = self.cache.current_period_text();

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
            CalendarView::Month => {
                // Use calendar cache module
                views::render_month_view(self.cache.current_state(), self.selected_day)
            },
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
