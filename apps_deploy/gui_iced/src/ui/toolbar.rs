use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length};

pub fn view_top_toolbar<'a>(
    is_loading: bool,
    status_message: &'a str,
) -> Element<'a, Message> {
    let reset_btn = button(text("Reset View").size(12))
        .style(style::ghost_button())
        .padding([4, 8])
        .on_press(Message::ResetView);

    let status_indicator = if is_loading {
        text("Loading...")
            .size(12)
            .style(style::ACCENT_AMBER)
    } else {
        text(status_message)
            .size(12)
            .style(style::TEXT_SECONDARY)
    };

    let title_badge = text("SAMDEF v0.1")
        .size(11)
        .style(style::TEXT_SECONDARY);

    container(
        row![
            reset_btn,
            Space::with_width(12),
            status_indicator,
            Space::with_width(Length::Fill),
            title_badge,
        ]
        .padding([6, 12])
        .align_items(Alignment::Center),
    )
    .width(Length::Fill)
    .style(style::toolbar_container())
    .into()
}


