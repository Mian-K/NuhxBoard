use crate::{message::Message, nuhxboard::NuhxBoard};
use iced::{
    Renderer, Theme,
    theme::Palette,
    widget::{center, text},
    window::Settings,
};
use iced_multi_window::Window;
use nuhxboard_types::style::Style;

#[derive(Debug)]
pub struct Main;

impl Window<NuhxBoard, Theme, Message> for Main {
    fn view<'a>(&'a self, _app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme, Renderer> {
        center(text!("droddyrox")).into()
    }

    fn title(&self, app: &NuhxBoard) -> String {
        app.settings.window_title.clone()
    }

    fn theme(&self, app: &NuhxBoard) -> Theme {
        Theme::custom(
            "NuhxBoard",
            Palette {
                background: app
                    .style
                    .as_ref()
                    .map(|s| s.background_color)
                    .unwrap_or_else(|| Style::default().background_color)
                    .into(),
                ..Palette::LIGHT
            },
        )
    }

    fn settings(&self) -> iced::window::Settings {
        Settings {
            exit_on_close_request: false,
            resizable: false,
            ..Default::default()
        }
    }
}
