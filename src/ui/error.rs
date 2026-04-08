use crate::{message::Message, nuhxboard::NuhxBoard};
use iced::{
    Alignment, Font, Length, Renderer, Theme,
    font::Weight,
    widget::{Text, center, column, text},
    window,
};
use iced_aw::Quad;
use iced_multi_window::Window;

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub details: String,
}

impl Window<NuhxBoard, Theme, Message> for Error {
    fn view<'a>(&'a self, _app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme, Renderer> {
        center(
            column![
                text!("Error: {}", self.message)
                    .font(Font {
                        weight: Weight::Bold,
                        ..Font::DEFAULT
                    })
                    .align_x(Alignment::Center),
                Quad {
                    height: 4.into(),
                    width: Length::Fill,
                    ..Default::default()
                },
                text!("Details:").font(Font {
                    weight: Weight::Semibold,
                    ..Default::default()
                }),
                Text::new(&self.details).size(15),
            ]
            .spacing(5)
            .align_x(Alignment::Center),
        )
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Error".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: (350, 200).into(),
            ..Default::default()
        }
    }
}
