pub mod class_filters;
pub mod file_nav;
pub mod quick_actions;
pub mod sidebar;
pub mod style;
pub mod telemetry;
pub mod toolbar;

use crate::app::{Message, SamdefViewer};
use crate::viewer::AnnotatedViewer;
use iced::widget::{column, container, horizontal_rule, row, text, vertical_rule, Space};
use iced::{Alignment, Element, Length};

pub fn view_app(app: &SamdefViewer) -> Element<'_, Message> {
    let sidebar = sidebar::view_sidebar(app);
    let main_viewer = view_canvas_container(app);

    row![
        container(sidebar)
            .width(Length::Fixed(340.0))
            .height(Length::Fill)
            .style(style::sidebar_container()),
        vertical_rule(1).style(style::dark_rule()),
        container(main_viewer)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::canvas_container()),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn view_canvas_container(app: &SamdefViewer) -> Element<'_, Message> {
    let top_toolbar = toolbar::view_top_toolbar(app.is_loading, &app.status_message);

    let viewer_content: Element<'_, Message> = if let Some(loaded_img) = &app.current_image {
        AnnotatedViewer::new(
            loaded_img.handle.clone(),
            &app.detections,
            app.class_visible,
            app.confidence_threshold,
            app.show_labels,
            app.reset_counter,
        )
        .on_status(|zoom, cursor| Message::ViewerStatus { zoom, cursor })
        .into()
    } else {
        container(
            column![
                text("No Satellite Image Loaded")
                    .size(18)
                    .style(style::TEXT_PRIMARY),
                Space::with_height(10),
                text("Select a JSON result from the sidebar or scan the results directory.")
                    .size(13)
                    .style(style::TEXT_SECONDARY),
            ]
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    };

    column![
        top_toolbar,
        horizontal_rule(1).style(style::dark_rule()),
        container(viewer_content)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

