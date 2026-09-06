pub mod message;
pub mod state;
pub mod update;

pub use message::Message;
pub use state::SamdefViewer;

use crate::ui::view_app;
use iced::{Application, Command, Element, Settings, Subscription, Theme};

impl Application for SamdefViewer {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let app = Self::default();
        let cmd = if let Some(path) = app.available_json_files.first().cloned() {
            update::load_json_command(path)
        } else {
            Command::none()
        };
        (app, cmd)
    }

    fn title(&self) -> String {
        let file_info = self
            .current_json_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("No file loaded");

        format!("SAMDEF GIS Viewer - [{}]", file_info)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        update::handle_update(self, message)
    }

    fn view(&self) -> Element<'_, Message> {
        view_app(self)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

pub fn run() -> iced::Result {
    println!("Starting SAMDEF GIS Viewer & Annotator...");
    SamdefViewer::run(Settings {
        window: iced::window::Settings {
            size: (1440, 920),
            position: iced::window::Position::Centered,
            min_size: Some((960, 640)),
            ..Default::default()
        },
        ..Default::default()
    })
}
