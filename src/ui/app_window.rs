use super::UiShard;
use crate::{app_context::AppContext, ui::generated};
use anyhow::Result;
use eventful_rs::*;
use slint::ComponentHandle;

use_shard!(shard = UiShard);

#[events]
pub trait AppEvents {
    fn on_first_time_setup_complete(&self);
}

#[eventful(AppEvents)]
pub struct App {
    ui: generated::AppWindow,
    app_ctx: AppContext,
}

impl App {
    pub fn new(ctx: AppContext) -> Result<ShardRc<Self>> {
        let ui = generated::AppWindow::new()?;
        Self::bind_local(Self {
            ui,
            app_ctx: ctx,
            events: Default::default(),
        })
        .map_err(|e| e.into())
    }

    pub fn show(&self) -> Result<()> {
        self.ui.show()?;
        Ok(())
    }

    pub fn quit(&self) {
        slint::quit_event_loop().unwrap();
    }
}
