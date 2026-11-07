use crate::app::{ActiveInput, SamdefApp};
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};

impl SamdefApp {
    pub fn render_file_nav(&self, cx: &Context<Self>) -> impl IntoElement {
        let query_lower = self.file_filter_query.trim().to_lowercase();
        let filtered_files: Vec<(usize, String)> = self
            .available_files
            .iter()
            .enumerate()
            .filter_map(|(idx, path)| {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                if query_lower.is_empty() || name.to_lowercase().contains(&query_lower) {
                    Some((idx, name))
                } else {
                    None
                }
            })
            .collect();

        let total_matches = filtered_files.len();
        let cur_name = self
            .current_file_idx
            .and_then(|i| self.available_files.get(i))
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Select inference file...");

        let dir_focused = self.active_input == Some(ActiveInput::ResultsDir);
        let search_focused = self.active_input == Some(ActiveInput::FileSearch);
        let status_left = match self.current_file_idx {
            Some(idx) => format!("[{}/{}] Loaded", idx + 1, self.available_files.len()),
            None => "No files loaded".to_string(),
        };

        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_SECONDARY))
                    .child("SOURCE DIRECTORY"),
            )
            // 1. Editable Directory Path Input + Scan Button
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1p5()
                    .child(
                        text_input_box(
                            "input-results-dir",
                            &self.results_dir,
                            "Path to results directory...",
                            dir_focused,
                            ACCENT_BLUE,
                        )
                        .flex_1()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.active_input = Some(ActiveInput::ResultsDir);
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        div()
                            .id("btn-scan-dir")
                            .px_2p5()
                            .py_1()
                            .rounded_sm()
                            .bg(rgb(ACCENT_BLUE))
                            .hover(|s| s.bg(rgb(0x2563eb)).cursor_pointer())
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(gpui::white())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.active_input = None;
                                    this.scan_directory(cx);
                                }),
                            )
                            .child("Scan"),
                    ),
            )
            // 2. Direct File Search / Filter Input (Enter or Go jumps immediately)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        text_input_box(
                            "input-file-search",
                            &self.file_filter_query,
                            "Jump to file (e.g. 1062)... [/]",
                            search_focused,
                            ACCENT_CYAN,
                        )
                        .flex_1()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.active_input = Some(ActiveInput::FileSearch);
                                this.file_dropdown_open = true;
                                cx.notify();
                            }),
                        ),
                    )
                    .children((!self.file_filter_query.is_empty()).then(|| {
                        action_btn("btn-clear-search", "X").on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.file_filter_query.clear();
                                cx.notify();
                            }),
                        )
                    }))
                    .child(action_btn("btn-go-search", "Go").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.submit_file_search(cx);
                        }),
                    )),
            )
            // 3. Closed-by-default Dropdown Trigger + Expandable Filtered Picker
            .child(
                div()
                    .id("btn-file-dropdown-toggle")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_2p5()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(BG_CARD))
                    .border_1()
                    .border_color(rgb(if self.file_dropdown_open {
                        ACCENT_BLUE
                    } else {
                        BORDER_STRONG
                    }))
                    .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.file_dropdown_open = !this.file_dropdown_open;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .overflow_hidden()
                            .child(div().size_2().rounded_full().bg(rgb(ACCENT_CYAN)))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(ACCENT_CYAN))
                                    .child(cur_name.to_string()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(if self.file_dropdown_open { "^" } else { "v" }),
                    ),
            )
            .children(self.file_dropdown_open.then(|| {
                div()
                    .id("file-selector-list")
                    .max_h(px(184.0))
                    .overflow_y_scroll()
                    .rounded_sm()
                    .bg(rgb(BG_INPUT))
                    .border_1()
                    .border_color(rgb(ACCENT_BLUE))
                    .flex()
                    .flex_col()
                    .children(if filtered_files.is_empty() {
                        vec![div()
                            .id("file-item-empty")
                            .px_2p5()
                            .py_1p5()
                            .text_xs()
                            .text_color(rgb(TEXT_DIM))
                            .child("No matching files")]
                    } else {
                        filtered_files
                            .into_iter()
                            .map(|(idx, name)| {
                                let is_active = self.current_file_idx == Some(idx);
                                div()
                                    .id(("file-item", idx))
                                    .px_2p5()
                                    .py_1()
                                    .border_b_1()
                                    .border_color(rgb(BORDER_SUBTLE))
                                    .bg(rgb(if is_active { 0x1e293b } else { BG_INPUT }))
                                    .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                                    .text_xs()
                                    .text_color(rgb(if is_active {
                                        ACCENT_BLUE
                                    } else {
                                        TEXT_PRIMARY
                                    }))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.file_dropdown_open = false;
                                            this.active_input = None;
                                            this.load_file_at_index(idx, cx);
                                        }),
                                    )
                                    .child(name)
                            })
                            .collect()
                    })
            }))
            // 4. Prev / Next Stepper + Status Counts
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(action_btn("btn-prev-file", "< Prev [P]").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_input = None;
                            this.prev_file(cx);
                        }),
                    ))
                    .child(action_btn("btn-next-file", "Next [N] >").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_input = None;
                            this.next_file(cx);
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(TEXT_DIM))
                    .child(status_left)
                    .child(format!("{} in list", total_matches)),
            )
    }
}
