use crate::app::CosmicCalendar;
use crate::components::{render_calendar_dialog, render_delete_calendar_dialog, render_event_dialog};
use crate::message::Message;
use crate::styles;
use crate::ui_constants::SIDEBAR_WIDTH;
use cosmic::iced::widget::stack;
use cosmic::iced::{alignment, Length};
use cosmic::widget::{container, divider, mouse_area, row};
use cosmic::Element;

/// Drag preview dimensions
const DRAG_PREVIEW_WIDTH: f32 = 150.0;
const DRAG_PREVIEW_HEIGHT: f32 = 24.0;
/// Offset from cursor to prevent preview from blocking interaction
const DRAG_PREVIEW_OFFSET_X: f32 = 10.0;
const DRAG_PREVIEW_OFFSET_Y: f32 = 10.0;

/// Render the responsive layout (sidebar + main content)
#[allow(deprecated)]
pub fn render_layout(app: &CosmicCalendar) -> Element<'_, Message> {
    let is_condensed = app.core.is_condensed();
    // Check active_dialog and legacy event_dialog for whether a dialog is open
    let has_dialog_open = app.active_dialog.is_open()
        || app.event_dialog.is_some();

    // Build base layout with sidebar inline when appropriate
    let base_content = if !is_condensed && app.show_sidebar {
        // Large screen: sidebar inline on left
        render_desktop_with_sidebar(app)
    } else if !is_condensed {
        // Large screen, sidebar hidden - wrap main content for dialog close
        wrap_main_content_for_dialog_close(app.render_main_content(), has_dialog_open)
    } else {
        // Condensed screen: just main content as base - wrap for dialog close
        wrap_main_content_for_dialog_close(app.render_main_content(), has_dialog_open)
    };

    // In condensed mode with sidebar toggled on, show it as overlay
    let with_sidebar = if is_condensed && app.show_sidebar {
        render_mobile_with_overlay(app, base_content)
    } else {
        base_content
    };

    // Show dialog overlays based on active_dialog state
    render_dialog_overlay(app, with_sidebar)
}

/// Render dialog overlay based on active_dialog state
/// Note: Event dialog still uses legacy field because text_editor::Content doesn't implement Clone
fn render_dialog_overlay<'a>(
    app: &'a CosmicCalendar,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    // First, add drag preview overlay if dragging an event
    let with_drag_preview = render_drag_preview_overlay(app, base);

    // Event dialog takes priority (uses legacy field due to text_editor::Content)
    #[allow(deprecated)]
    if let Some(ref dialog_state) = app.event_dialog {
        let dialog = render_event_dialog(dialog_state, app.calendar_manager.sources());
        return stack![with_drag_preview, dialog].into();
    }

    // Check active_dialog for calendar dialogs
    use crate::dialogs::ActiveDialog;
    match &app.active_dialog {
        ActiveDialog::CalendarCreate { .. } | ActiveDialog::CalendarEdit { .. } => {
            let dialog = render_calendar_dialog(&app.active_dialog);
            return stack![with_drag_preview, dialog].into();
        }
        ActiveDialog::CalendarDelete { .. } => {
            let dialog = render_delete_calendar_dialog(&app.active_dialog);
            return stack![with_drag_preview, dialog].into();
        }
        _ => {}
    }

    with_drag_preview
}

/// Render a floating drag preview overlay when an event is being dragged
fn render_drag_preview_overlay<'a>(
    app: &'a CosmicCalendar,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    use cosmic::widget::text;
    use cosmic::iced::Background;

    let drag_state = &app.event_drag_state;

    // Only show preview if actively dragging and we have cursor position
    if !drag_state.is_active {
        return base;
    }

    let Some((cursor_x, cursor_y)) = drag_state.cursor_position() else {
        return base;
    };

    let Some(summary) = drag_state.event_summary() else {
        return base;
    };

    let Some(color_hex) = drag_state.event_color() else {
        return base;
    };

    // Parse the color
    let color = crate::components::parse_hex_color(color_hex)
        .unwrap_or(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5));

    // Create the drag preview chip - styled similar to event chips
    let preview_content = text(summary)
        .size(11)
        .width(Length::Fill);

    let preview_chip = container(preview_content)
        .padding([4, 8, 4, 8])
        .width(Length::Fixed(DRAG_PREVIEW_WIDTH))
        .height(Length::Fixed(DRAG_PREVIEW_HEIGHT))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(Background::Color(color.scale_alpha(0.8))),
                text_color: Some(cosmic::iced::Color::WHITE),
                border: cosmic::iced::Border {
                    color: color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: cosmic::iced::Shadow {
                    color: cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: cosmic::iced::Vector::new(2.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            }
        });

    // Position the preview at cursor location with offset
    // Use a container positioned at (0,0) with padding to simulate absolute positioning
    let preview_x = cursor_x + DRAG_PREVIEW_OFFSET_X;
    let preview_y = cursor_y + DRAG_PREVIEW_OFFSET_Y;

    // Wrap in a container that fills the screen and positions the preview
    let positioned_preview = container(preview_chip)
        .padding([preview_y as u16, 0, 0, preview_x as u16])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Top);

    stack![base, positioned_preview].into()
}

/// Wrap main content with a mouse_area to close dialogs when clicking outside
fn wrap_main_content_for_dialog_close<'a>(
    content: Element<'a, Message>,
    dialog_open: bool,
) -> Element<'a, Message> {
    if dialog_open {
        mouse_area(content)
            .on_press(Message::CloseDialog)
            .into()
    } else {
        content
    }
}

/// Render desktop layout with inline sidebar
fn render_desktop_with_sidebar(app: &CosmicCalendar) -> Element<'_, Message> {
    let main_content = wrap_main_content_for_dialog_close(
        app.render_main_content(),
        app.active_dialog.is_open(),
    );

    row()
        .spacing(0)
        .push(app.render_sidebar())
        .push(divider::vertical::default())
        .push(main_content)
        .into()
}

/// Render mobile layout with overlay sidebar
fn render_mobile_with_overlay<'a>(
    app: &'a CosmicCalendar,
    base_content: Element<'a, Message>,
) -> Element<'a, Message> {
    let overlay_sidebar = container(
        container(app.render_sidebar()).style(styles::overlay_sidebar_style),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Left);

    stack![base_content, overlay_sidebar].into()
}
