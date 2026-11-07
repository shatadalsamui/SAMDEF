use crate::app::{ActiveInput, SamdefApp};
use gpui::{Context, KeystrokeEvent};

impl SamdefApp {
    /// Jump to and load the first file matching `self.file_filter_query` (mirrors `gui_iced`)
    pub fn submit_file_search(&mut self, cx: &mut Context<Self>) {
        let query = self.file_filter_query.trim().to_lowercase();
        let first_match = self.available_files.iter().position(|path| {
            if query.is_empty() {
                true
            } else {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.to_lowercase().contains(&query))
                    .unwrap_or(false)
            }
        });

        if let Some(idx) = first_match {
            self.file_dropdown_open = false;
            self.active_input = None;
            self.load_file_at_index(idx, cx);
        } else {
            self.status_message = format!("No file matching '{}'", self.file_filter_query.trim());
            cx.notify();
        }
    }

    /// Handle typing, clipboard paste, backspace, Enter submit, and Escape when a text input is focused
    pub fn handle_input_keystroke(&mut self, ev: &KeystrokeEvent, cx: &mut Context<Self>) -> bool {
        let Some(target) = self.active_input else {
            return false;
        };

        let key = ev.keystroke.key.as_str();
        let mods = &ev.keystroke.modifiers;

        match key {
            "escape" => {
                self.active_input = None;
                self.file_dropdown_open = false;
                cx.notify();
                return true;
            }
            "enter" => {
                match target {
                    ActiveInput::ResultsDir => {
                        self.active_input = None;
                        self.scan_directory(cx);
                    }
                    ActiveInput::FileSearch => {
                        self.submit_file_search(cx);
                    }
                    ActiveInput::TableSearch => {
                        self.active_input = None;
                        cx.notify();
                    }
                }
                return true;
            }
            "backspace" => {
                let buf = self.input_buffer_mut(target);
                if mods.control {
                    buf.clear();
                } else {
                    buf.pop();
                }
                if target == ActiveInput::FileSearch {
                    self.file_dropdown_open = true;
                }
                cx.notify();
                return true;
            }
            "v" if mods.control || mods.platform => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    let clean = text.trim().replace(['\n', '\r'], "");
                    self.input_buffer_mut(target).push_str(&clean);
                    if target == ActiveInput::FileSearch {
                        self.file_dropdown_open = true;
                    }
                    cx.notify();
                }
                return true;
            }
            "u" if mods.control => {
                self.input_buffer_mut(target).clear();
                cx.notify();
                return true;
            }
            _ => {}
        }

        if !mods.control
            && !mods.alt
            && !mods.platform
            && let Some(ch) = &ev.keystroke.key_char
            && !ch.is_empty()
        {
            self.input_buffer_mut(target).push_str(ch);
            if target == ActiveInput::FileSearch {
                self.file_dropdown_open = true;
            }
            cx.notify();
            return true;
        }

        true
    }

    fn input_buffer_mut(&mut self, target: ActiveInput) -> &mut String {
        match target {
            ActiveInput::ResultsDir => &mut self.results_dir,
            ActiveInput::FileSearch => &mut self.file_filter_query,
            ActiveInput::TableSearch => &mut self.table_search_query,
        }
    }
}

