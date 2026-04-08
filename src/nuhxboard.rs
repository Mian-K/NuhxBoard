use crate::{KEYBOARDS_PATH, message::Message, ui};
use futures::{StreamExt, TryStreamExt};
use iced::{
    Event, Task, Theme,
    window::{self, Id},
};
use iced_multi_window::WindowManager;
use nuhxboard_types::{layout::Layout, settings::Settings, style::Style};
use tracing::{debug, error, info, trace};

pub struct NuhxBoard {
    pub startup: bool,
    pub main_window: Id,
    pub settings: Settings,
    pub windows: WindowManager<Self, Theme, Message>,
    pub layout_options: Vec<String>,
    pub layout: Option<Layout>,
    pub style: Option<Style>,
}

/// Try a result, returning `Some(Err(e))` if it fails.
macro_rules! try_or {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(e) => e,
            ::std::result::Result::Err(e) => {
                return ::std::option::Option::Some(::std::result::Result::Err(e))
            }
        }
    };
}

macro_rules! send {
    ($e:expr) => {
        Task::perform(std::future::ready(()), |_| $e)
    };
}

impl NuhxBoard {
    pub fn new() -> (Self, Task<Message>) {
        let mut windows = WindowManager::default();
        let mut tasks = Vec::new();

        let settings = confy::load::<Settings>("NuhxBoard", None).unwrap_or_else(|e| {
            tasks.push(send!(Message::error(
                "Failed to load settings. Using defaults.",
                e
            )));
            Settings::default()
        });

        let (main_window, main_window_task) = windows.open(ui::Main);
        tasks.push(main_window_task.map(|_| Message::LoadCategory));

        (
            Self {
                startup: true,
                main_window,
                settings,
                windows,
                layout: None,
                style: None,
                layout_options: Vec::new(),
            },
            Task::batch(tasks),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        trace!(?message);
        match message {
            Message::Ignore => {}
            Message::CloseRequested(id) => {
                if self.main_window == id {
                    return self.exit();
                }
            }
            Message::Closed(id) => {
                self.windows.was_closed(id);
                if self.windows.empty() {
                    return self.exit();
                }
            }
            Message::Error {
                message,
                details,
                debug: dbg,
            } => {
                error!(error = dbg, "Error: {message}");
                return self.open_window(ui::Error { message, details });
            }
            Message::LoadCategory => {
                let Some(category) = self.settings.category.clone() else {
                    return Task::none();
                };
                return Task::perform(
                    async move {
                        smol::fs::read_dir(KEYBOARDS_PATH.join(category))
                            .await?
                            .filter_map(async |entry| {
                                let entry = try_or!(entry);

                                if try_or!(entry.file_type().await).is_dir()
                                    && entry.file_name() != "images"
                                {
                                    match entry.file_name().into_string() {
                                        Ok(name) => Some(Ok(name)),
                                        Err(name) => {
                                            error!(
                                                ?name,
                                                "Failed to convert directory name to string"
                                            );
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                            .try_collect::<Vec<_>>()
                            .await
                    },
                    |r| match r {
                        Ok(v) => Message::CategoryLoaded(v),
                        Err(e) => Message::error("Failed to load category.", e),
                    },
                );
            }
            Message::CategoryLoaded(layout_options) => {
                info!(
                    "Loaded category: {}",
                    self.settings
                        .category
                        .as_ref()
                        .expect("CategoryLoaded with no category")
                );
                debug!(?layout_options);
                self.layout_options = layout_options;
                if self.startup {
                    return self.update(Message::LoadLayout);
                }
            }
            Message::LoadLayout => {
                let Some(category) = self.settings.category.clone() else {
                    return Task::none();
                };
                let Some(index) = self.settings.layout_index else {
                    return Task::none();
                };
                let layout_name = self
                    .layout_options
                    .get(index)
                    .expect("Layout index out of bounds")
                    .clone();

                return Task::perform(
                    smol::unblock(move || {
                        Ok(serde_json::from_reader(std::fs::File::open(
                            KEYBOARDS_PATH
                                .join(category)
                                .join(layout_name)
                                .join("keyboard.json"),
                        )?)?)
                    }),
                    |r: std::io::Result<Layout>| match r {
                        Ok(layout) => Message::LayoutLoaded(layout),
                        Err(e) => Message::error("Failed to load layout.", e),
                    },
                );
            }
            Message::LayoutLoaded(layout) => {
                info!(
                    "Loaded layout: {}",
                    self.layout_options
                        .get(
                            self.settings
                                .layout_index
                                .expect("LayoutLoaded with no layout_index")
                        )
                        .expect("Layout index out of bounds")
                );
                let size = (layout.width, layout.height).into();
                self.layout = Some(layout);
                if std::env::var("XDG_SESSION_TYPE").is_ok_and(|s| s == "wayland") {
                    // window::resize just... doesn't work? on wayland? or maybe just niri? idfk
                    // this is a fine workaround
                    return window::set_min_size(self.main_window, Some(size))
                        .chain(window::set_max_size(self.main_window, Some(size)));
                } else {
                    return window::resize(self.main_window, size);
                }
            }
        }

        Task::none()
    }

    fn open_window(
        &mut self,
        window: impl iced_multi_window::Window<Self, Theme, Message> + 'static,
    ) -> Task<Message> {
        self.windows.open(window).1.map(|_| Message::Ignore)
    }

    // TODO: cleanup
    fn exit(&mut self) -> Task<Message> {
        let settings = self.settings.clone();
        Task::batch([
            Task::perform(
                smol::unblock(|| confy::store("NuhxBoard", None, settings)),
                |r| {
                    if let Err(error) = r {
                        error!(?error, "Failed to save settings");
                    }
                    Message::Ignore
                },
            ),
            iced::exit(),
        ])
    }

    pub fn view(&self, id: Id) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        self.windows.view(self, id)
    }

    pub fn theme(&self, id: Id) -> Theme {
        self.windows.theme(self, id)
    }

    pub fn title(&self, id: Id) -> String {
        self.windows.title(self, id)
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|e, _, id| match e {
            Event::Window(window::Event::CloseRequested) => Some(Message::CloseRequested(id)),
            Event::Window(window::Event::Closed) => Some(Message::Closed(id)),
            _ => None,
        })
    }
}
