use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, column, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChoice {
    pub idx: usize,
    pub name: String,
}

impl std::fmt::Display for FileChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn view_file_navigator<'a>(
    results_dir_input: &'a str,
    file_filter_query: &'a str,
    available_files: &'a [PathBuf],
    current_idx: Option<usize>,
) -> Element<'a, Message> {
    let header = text("SOURCE DIRECTORY")
        .size(11)
        .style(style::TEXT_HEADER);

    let path_row = row![
        text_input("Path to results directory...", results_dir_input)
            .on_input(Message::ResultsDirChanged)
            .on_submit(Message::ScanResultsDir)
            .size(12)
            .padding(6)
            .width(Length::Fill)
            .style(style::dark_input()),
        button(text("Scan").size(12))
            .style(style::ghost_button_accent())
            .padding([4, 8])
            .on_press(Message::ScanResultsDir),
    ]
    .spacing(6)
    .align_items(Alignment::Center);

    // Direct search/filter input for choosing by name
    let search_input = text_input("Jump to file (e.g. 1062)...", file_filter_query)
        .on_input(Message::FileFilterChanged)
        .on_submit(Message::FileFilterSubmitted)
        .size(12)
        .padding(6)
        .style(style::dark_input());

    // Filter choices according to search query
    let query = file_filter_query.trim().to_lowercase();
    let choices: Vec<FileChoice> = available_files
        .iter()
        .enumerate()
        .filter(|(_, path)| {
            if query.is_empty() {
                true
            } else {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.to_lowercase().contains(&query))
                    .unwrap_or(false)
            }
        })
        .map(|(idx, path)| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            FileChoice { idx, name }
        })
        .collect();

    let selected_choice = current_idx.and_then(|idx| {
        available_files.get(idx).map(|path| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            FileChoice { idx, name }
        })
    });

    let total_matches = choices.len();

    // Closed-by-default dropdown menu
    let dropdown = iced::widget::pick_list(
        choices,
        selected_choice,
        |choice: FileChoice| Message::SelectJsonFile(choice.idx),
    )
    .placeholder("Select inference file...")
    .width(Length::Fill)
    .padding(6)
    .text_size(12);

    // Stepper navigation buttons
    let nav_buttons = row![
        button(text("< Prev").size(12))
            .style(style::ghost_button())
            .padding([4, 8])
            .on_press(Message::PrevFile),
        Space::with_width(Length::Fill),
        button(text("Next >").size(12))
            .style(style::ghost_button())
            .padding([4, 8])
            .on_press(Message::NextFile),
    ]
    .align_items(Alignment::Center);

    let current_file_label = if let Some(idx) = current_idx {
        format!("[{}/{}] Loaded", idx + 1, available_files.len())
    } else {
        "No files loaded".to_string()
    };

    let status_row = row![
        text(current_file_label)
            .size(11)
            .style(style::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} in list", total_matches))
            .size(11)
            .style(style::TEXT_SECONDARY),
    ];

    column![
        header,
        path_row,
        search_input,
        dropdown,
        nav_buttons,
        status_row
    ]
    .spacing(6)
    .into()
}
