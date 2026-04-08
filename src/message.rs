use iced::window::Id;
use nuhxboard_types::layout::Layout;

#[derive(Debug, Clone)]
pub enum Message {
    CloseRequested(Id),
    Closed(Id),
    LoadCategory,
    CategoryLoaded(Vec<String>),
    LoadLayout,
    LayoutLoaded(Layout),
    Error {
        message: String,
        details: String,
        debug: String,
    },
    Ignore,
}

impl Message {
    pub fn error(message: impl Into<String>, error: impl std::error::Error) -> Self {
        Self::Error {
            message: message.into(),
            details: error.to_string(),
            debug: format!("{:#?}", error),
        }
    }
}
