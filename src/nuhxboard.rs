use crate::{KEYBOARDS_PATH, ui::ErrorPopup};
use async_channel::Receiver;
use futures::{StreamExt, TryStreamExt};
use gpui::{AppContext, AsyncApp, Context, ParentElement, Render, Styled, WeakEntity, div, rgb};
use nuhxboard_types::{layout::Layout, settings::Settings};
use rdevin::Event;
use std::rc::Rc;
use tracing::error;

#[derive(Debug)]
pub struct NuhxBoard {
    settings: Settings,
    category: Option<Rc<str>>,
    layout_options: Vec<Rc<str>>,
    layout: Layout,
}

impl NuhxBoard {
    pub fn new(ctx: &mut Context<Self>, rx: Receiver<Event>) -> Self {
        ctx.spawn(async move |this, app| {
            while let Ok(event) = rx.recv().await {
                this.update(app, |this, ctx| {
                    this.handle_event(event);
                    ctx.notify();
                })
                .unwrap();
            }
        })
        .detach();

        let settings: Settings = confy::load("NuhxBoard", None).unwrap_or_else(|error| {
            error!(?error, "failed to load settings");
            Default::default()
        });

        if let Some(category) = settings.category.clone() {
            Self::execute_fallible_async(ctx, async move |this, app| {
                Self::set_category(this, app, category).await
            });
        }

        Self::execute_fallible_async(ctx, async move |this, app| {
            Self::set_layout(this, app, settings.layout_index).await
        });

        Self {
            category: settings.category.clone(),
            settings,
            layout_options: Vec::new(),
            layout: Layout::default(),
        }
    }

    fn handle_event(&mut self, event: Event) {
        todo!()
    }

    async fn set_category(
        this: WeakEntity<Self>,
        app: &mut AsyncApp,
        category: Rc<str>,
    ) -> anyhow::Result<()> {
        let mut layout_options = async_fs::read_dir(KEYBOARDS_PATH.join(category.as_ref()))
            .await?
            .then(async |entry| {
                let entry = entry?;
                if entry.file_type().await?.is_dir() && entry.file_name() != "images" {
                    std::io::Result::Ok(Some(
                        entry
                            .file_name()
                            .to_str()
                            .expect("invalid utf-8 file name")
                            .into(),
                    ))
                } else {
                    Ok(None)
                }
            })
            .map(Result::transpose)
            .filter_map(async |x| x)
            .try_collect::<Vec<_>>()
            .await?;

        layout_options.sort();

        this.update(app, |this, ctx| {
            this.layout_options = layout_options;
            this.category = Some(category);
            ctx.notify();
        })?;

        Ok(())
    }

    async fn set_layout(
        this: WeakEntity<Self>,
        app: &mut AsyncApp,
        index: usize,
    ) -> anyhow::Result<()> {
        let (Some(category), Some(layout_name)) = this.read_with(app, |this, _| {
            (
                this.category.clone(),
                this.layout_options.get(index).cloned(),
            )
        })?
        else {
            return Ok(());
        };

        let layout = serde_json::from_str(
            &async_fs::read_to_string(
                KEYBOARDS_PATH
                    .join(category.as_ref())
                    .join(layout_name.as_ref()),
            )
            .await?,
        )?;

        this.update(app, |this, ctx| {
            this.layout = layout;
            ctx.notify();
        })?;

        Ok(())
    }

    fn execute_fallible_async(
        ctx: &mut Context<'_, Self>,
        f: impl AsyncFnOnce(WeakEntity<Self>, &mut AsyncApp) -> anyhow::Result<()> + 'static,
    ) {
        ctx.spawn(async move |this, app| {
            if let Err(error) = f(this, app).await {
                app.open_window(Default::default(), |_, app| {
                    app.new(|_| ErrorPopup::new(error))
                })
                .unwrap();
            }
        })
        .detach();
    }
}

impl Render for NuhxBoard {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        tracing::debug!(?self);
        div()
            .bg(rgb(0xffffff))
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(format!("Category: {self:#?}"))
    }
}
