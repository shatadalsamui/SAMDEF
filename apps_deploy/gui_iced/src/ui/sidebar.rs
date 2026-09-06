use crate::app::{Message, SamdefViewer};
use crate::ui::class_filters::view_class_filters;
use crate::ui::file_nav::view_file_navigator;
use crate::ui::quick_actions::view_quick_actions;
use crate::ui::style;
use crate::ui::telemetry::view_telemetry;
use iced::widget::{column, horizontal_rule, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub fn view_sidebar(app: &SamdefViewer) -> Element<'_, Message> {
    let title_header = column![
        row![
            text("SAMDEF").size(15).style(style::TEXT_PRIMARY),
            text("GIS VIEWER").size(13).style(style::TEXT_SECONDARY),
        ]
        .spacing(6)
        .align_items(Alignment::Center),
        text("Satellite Object Detection & Exploitation")
            .size(11)
            .style(style::TEXT_SECONDARY),
    ]
    .spacing(3);

    let file_nav = view_file_navigator(
        &app.results_dir_input,
        &app.file_filter_query,
        &app.available_json_files,
        app.current_file_idx,
    );

    let quick_actions = view_quick_actions(app.show_labels);

    let class_counts = app.class_counts();
    let class_filters = view_class_filters(
        &app.class_visible,
        &class_counts,
        app.confidence_threshold,
    );

    let img_dims = app
        .current_image
        .as_ref()
        .map(|img| (img.width, img.height))
        .unwrap_or((0, 0));

    let telemetry = view_telemetry(
        img_dims,
        app.count_visible_detections(),
        app.detections.len(),
        app.current_zoom,
        app.cursor_coord,
    );

    let content = column![
        title_header,
        horizontal_rule(1).style(style::dark_rule()),
        file_nav,
        horizontal_rule(1).style(style::dark_rule()),
        quick_actions,
        horizontal_rule(1).style(style::dark_rule()),
        class_filters,
        horizontal_rule(1).style(style::dark_rule()),
        telemetry,
        Space::with_height(16),
    ]
    .spacing(12)
    .padding(14);

    scrollable(content).height(Length::Fill).into()
}
