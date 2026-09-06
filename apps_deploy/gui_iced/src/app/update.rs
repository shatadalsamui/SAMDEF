use crate::app::message::Message;
use crate::app::state::SamdefViewer;
use crate::io::{load_tiff_or_image, resolve_tiff_path, scan_json_directory};
use crate::model::FinalOutput;
use iced::Command;
use std::fs;
use std::path::PathBuf;

pub fn handle_update(app: &mut SamdefViewer, message: Message) -> Command<Message> {
    match message {
        Message::ResultsDirChanged(dir) => {
            app.results_dir_input = dir;
            Command::none()
        }

        Message::ScanResultsDir => {
            match scan_json_directory(&app.results_dir_input) {
                Ok(files) => {
                    app.status_message = format!("Found {} JSON result files", files.len());
                    app.available_json_files = files;
                    if !app.available_json_files.is_empty() {
                        app.current_file_idx = Some(0);
                        let first = app.available_json_files[0].clone();
                        load_json_command(first)
                    } else {
                        Command::none()
                    }
                }
                Err(err) => {
                    app.status_message = err;
                    Command::none()
                }
            }
        }

        Message::FileFilterChanged(query) => {
            app.file_filter_query = query;
            Command::none()
        }

        Message::FileFilterSubmitted => {
            let query = app.file_filter_query.trim().to_lowercase();
            let first_match = app.available_json_files.iter().position(|path| {
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
                if let Some(path) = app.available_json_files.get(idx).cloned() {
                    app.current_file_idx = Some(idx);
                    load_json_command(path)
                } else {
                    Command::none()
                }
            } else {
                Command::none()
            }
        }

        Message::SelectJsonFile(idx) => {
            if let Some(path) = app.available_json_files.get(idx).cloned() {
                app.current_file_idx = Some(idx);
                load_json_command(path)
            } else {
                Command::none()
            }
        }

        Message::NextFile => {
            if !app.available_json_files.is_empty() {
                let next_idx = match app.current_file_idx {
                    Some(idx) => (idx + 1) % app.available_json_files.len(),
                    None => 0,
                };
                app.current_file_idx = Some(next_idx);
                let path = app.available_json_files[next_idx].clone();
                load_json_command(path)
            } else {
                Command::none()
            }
        }

        Message::PrevFile => {
            if !app.available_json_files.is_empty() {
                let prev_idx = match app.current_file_idx {
                    Some(idx) => {
                        if idx == 0 {
                            app.available_json_files.len() - 1
                        } else {
                            idx - 1
                        }
                    }
                    None => 0,
                };
                app.current_file_idx = Some(prev_idx);
                let path = app.available_json_files[prev_idx].clone();
                load_json_command(path)
            } else {
                Command::none()
            }
        }

        Message::JsonLoaded(result) => {
            match result {
                Ok((json_path, output)) => {
                    app.current_json_path = Some(json_path.clone());
                    app.detections = output.detections.iter().map(|d| d.into()).collect();
                    app.status_message = format!(
                        "Loaded {} ({} detections). Decoding TIFF...",
                        json_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                        app.detections.len()
                    );

                    let img_path = resolve_tiff_path(&json_path, &output.source_image);
                    app.is_loading = true;

                    Command::perform(
                        async move {
                            load_tiff_or_image(&img_path)
                                .map_err(|e| format!("Failed to load TIFF at {}: {}", img_path.display(), e))
                        },
                        Message::ImageLoaded,
                    )
                }
                Err(err) => {
                    app.is_loading = false;
                    app.status_message = format!("Error reading JSON: {}", err);
                    Command::none()
                }
            }
        }

        Message::ImageLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(img) => {
                    app.status_message = format!("Ready: {}x{} | {} detections", img.width, img.height, app.detections.len());
                    app.current_image = Some(img);
                    app.reset_counter += 1;
                    app.current_zoom = 1.0;
                }
                Err(err) => { app.status_message = err; }
            }
            Command::none()
        }

        Message::ToggleClass(class_id, val) => {
            if class_id < app.class_visible.len() {
                app.class_visible[class_id] = val;
            }
            Command::none()
        }

        Message::SelectAllClasses => {
            app.class_visible = [true; 8];
            app.status_message = "Filter: All classes selected".to_string();
            Command::none()
        }

        Message::TurnOffAllClasses => {
            app.class_visible = [false; 8];
            app.status_message = "Filter: All annotations turned off".to_string();
            Command::none()
        }

        Message::ConfidenceChanged(conf) => {
            app.confidence_threshold = conf;
            Command::none()
        }

        Message::ToggleLabels(val) => {
            app.show_labels = val;
            Command::none()
        }

        Message::ViewerStatus { zoom, cursor } => {
            app.current_zoom = zoom;
            app.cursor_coord = cursor;
            Command::none()
        }

        Message::ResetView => {
            app.reset_counter += 1;
            app.current_zoom = 1.0;
            Command::none()
        }
    }
}

pub fn load_json_command(path: PathBuf) -> Command<Message> {
    Command::perform(
        async move {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let output: FinalOutput = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
            Ok((path, output))
        },
        Message::JsonLoaded,
    )
}
