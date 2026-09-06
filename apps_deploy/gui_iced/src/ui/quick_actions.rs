use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, checkbox, column, row, text};
use iced::{Element, Length};


pub fn view_quick_actions<'a>(
    show_labels: bool,
) -> Element<'a, Message> {
    let header = text("ANNOTATION CONTROLS")
        .size(11)
        .style(style::TEXT_HEADER);

    let batch_buttons = row![
        button(text("Select All").size(12))
            .style(style::ghost_button())
            .width(Length::FillPortion(1))
            .padding([4, 6])
            .on_press(Message::SelectAllClasses),
        button(text("Clear All").size(12))
            .style(style::ghost_button())
            .width(Length::FillPortion(1))
            .padding([4, 6])
            .on_press(Message::TurnOffAllClasses),
    ]
    .spacing(8);

    let badge_toggle = checkbox(
        "Show Class & Confidence Badges",
        show_labels,
        Message::ToggleLabels,
    )
    .size(15)
    .style(style::vscode_checkbox());

    column![header, batch_buttons, badge_toggle]
        .spacing(8)
        .into()
}
