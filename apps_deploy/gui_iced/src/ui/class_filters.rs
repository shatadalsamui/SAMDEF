use crate::app::Message;
use crate::model::{class_color, class_name};
use crate::ui::style;
use iced::widget::{checkbox, column, container, row, slider, text, Space};
use iced::{Alignment, Element, Length};

pub fn view_class_filters<'a>(
    class_visible: &[bool; 8],
    class_counts: &[usize; 8],
    confidence_threshold: f32,
) -> Element<'a, Message> {
    let header = text("TARGET CLASSES")
        .size(11)
        .style(style::TEXT_HEADER);

    let mut class_rows = column![].spacing(6);
    for i in 0..8 {
        let color = class_color(i);
        let name = class_name(i);
        let count = class_counts[i];
        let is_checked = class_visible[i];

        let count_style = if count > 0 {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_SECONDARY
        };

        // Real colored rectangle swatch instead of font glyph (prevents missing glyph boxes)
        let swatch = container(Space::new(Length::Fixed(10.0), Length::Fixed(10.0)))
            .style(style::color_swatch(color));

        let item = row![
            checkbox("", is_checked, move |val| Message::ToggleClass(i, val))
                .size(14)
                .style(style::vscode_checkbox()),
            swatch,
            text(name)
                .size(12)
                .width(Length::Fill)
                .style(style::TEXT_PRIMARY),
            text(format!("{}", count))
                .size(11)
                .style(count_style),
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        class_rows = class_rows.push(item);
    }

    let conf_header = row![
        text("Confidence Threshold").size(11).style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{:.0}%", confidence_threshold * 100.0))
            .size(11)
            .style(style::TEXT_PRIMARY),
    ];

    let conf_slider = slider(
        0.0..=1.0,
        confidence_threshold,
        Message::ConfidenceChanged,
    )
    .step(0.01)
    .style(style::vscode_slider());

    column![header, class_rows, Space::with_height(4), conf_header, conf_slider]
        .spacing(8)
        .into()
}


