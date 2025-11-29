use crate::app::CosmicCalendar;
use crate::components::{render_calendar_dialog, render_delete_calendar_dialog};
use crate::message::Message;
use crate::styles;
use crate::ui_constants::SIDEBAR_WIDTH;
use cosmic::iced::widget::stack;
use cosmic::iced::{alignment, Length};
use cosmic::widget::{container, divider, mouse_area, row};
use cosmic::Element;

/// Render the responsive layout (sidebar + main content)
pub fn render_layout(app: &CosmicCalendar) -> Element<'_, Message> {
    let is_condensed = app.core.is_condensed();
    let color_picker_open = app.color_picker_open.is_some();

    // Build base layout with sidebar inline when appropriate
    let base_content = if !is_condensed && app.show_sidebar {
        // Large screen: sidebar inline on left
        render_desktop_with_sidebar(app)
    } else if !is_condensed {
        // Large screen, sidebar hidden - wrap main content for color picker close
        wrap_main_content_for_color_picker(app.render_main_content(), color_picker_open)
    } else {
        // Condensed screen: just main content as base - wrap for color picker close
        wrap_main_content_for_color_picker(app.render_main_content(), color_picker_open)
    };

    // In condensed mode with sidebar toggled on, show it as overlay
    let with_sidebar = if is_condensed && app.show_sidebar {
        render_mobile_with_overlay(app, base_content)
    } else {
        base_content
    };

    // Show dialog overlays if any are open
    if let Some(ref dialog_state) = app.calendar_dialog {
        let dialog = render_calendar_dialog(dialog_state);
        return stack![with_sidebar, dialog].into();
    }

    if let Some(ref dialog_state) = app.delete_calendar_dialog {
        let dialog = render_delete_calendar_dialog(dialog_state);
        return stack![with_sidebar, dialog].into();
    }

    with_sidebar
}

/// Wrap main content with a mouse_area to close color picker when clicking outside
fn wrap_main_content_for_color_picker<'a>(
    content: Element<'a, Message>,
    color_picker_open: bool,
) -> Element<'a, Message> {
    if color_picker_open {
        mouse_area(content)
            .on_press(Message::CloseColorPicker)
            .into()
    } else {
        content
    }
}

/// Render desktop layout with inline sidebar
fn render_desktop_with_sidebar(app: &CosmicCalendar) -> Element<'_, Message> {
    let main_content = wrap_main_content_for_color_picker(
        app.render_main_content(),
        app.color_picker_open.is_some(),
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
