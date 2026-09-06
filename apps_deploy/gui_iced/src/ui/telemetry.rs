use crate::app::Message;
use crate::ui::style;
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

pub fn view_telemetry<'a>(
    img_dims: (u32, u32),
    visible_count: usize,
    total_count: usize,
    zoom: f32,
    cursor_coord: Option<(f32, f32)>,
) -> Element<'a, Message> {
    let header = text("TELEMETRY & STATS")
        .size(11)
        .style(style::TEXT_HEADER);

    let res_row = row![
        text("Resolution").size(11).style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} × {} px", img_dims.0, img_dims.1))
            .size(11)
            .style(style::TEXT_PRIMARY),
    ];

    let count_row = row![
        text("Detections").size(11).style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} / {}", visible_count, total_count))
            .size(11)
            .style(style::ACCENT_GREEN),
    ];

    let zoom_row = row![
        text("Zoom Level").size(11).style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{:.0}%", zoom * 100.0))
            .size(11)
            .style(style::TEXT_PRIMARY),
    ];

    let cursor_row = row![
        text("Cursor Coordinates").size(11).style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(match cursor_coord {
            Some((x, y)) => format!("X:{:.0} Y:{:.0}", x, y),
            None => "Outside Image".to_string(),
        })
        .size(11)
        .style(if cursor_coord.is_some() {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_SECONDARY
        }),
    ];

    let card = container(
        column![res_row, count_row, zoom_row, cursor_row]
            .spacing(6)
            .padding(10),
    )
    .width(Length::Fill)
    .style(style::card_container());

    column![header, card].spacing(8).into()
}


