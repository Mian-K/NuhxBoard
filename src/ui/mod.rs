use gpui::{FontWeight, ParentElement, Render, Styled, div, rgb};

pub struct ErrorPopup(anyhow::Error);

impl ErrorPopup {
    pub fn new(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl Render for ErrorPopup {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .bg(rgb(0xffffff))
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(div().child("Error").font_weight(FontWeight::BOLD))
            .child(self.0.to_string())
    }
}
