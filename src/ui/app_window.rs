use std::sync::Arc;

use super::UiShard;
use crate::services::ncclient::*;
use crate::{app_context::AppContext, models::config::config, ui::generated};
use anyhow::Result;
use eventful_rs::*;
use slint::{CloseRequestResponse, ComponentHandle};

use_shard!(shard = UiShard);

#[slint_events(component = generated::AppWindow)]
trait AppUiEvents {}

#[events]
pub trait AppEvents {
    fn on_first_time_setup_complete(&self);
}

#[eventful(AppEvents)]
pub struct App {
    ui: generated::AppWindow,
    app_ctx: Arc<AppContext>,
    ui_events: AppUiEventsBridge,
}

impl App {
    pub fn new(ctx: Arc<AppContext>) -> Result<ShardRc<Self>> {
        let ui = generated::AppWindow::new()?;
        let w = Self::bind_local(Self {
            app_ctx: ctx,
            ui_events: AppUiEventsBridge::new(&ui),
            ui,
            events: Default::default(),
        })?;

        w.ui.set_is_loading(true);
        w.ui.set_loading_status(
            format!(
                "Authenticating with {}...",
                config()
                    .nextcloud_server
                    .map(|url| url.to_string())
                    .unwrap_or_default()
            )
            .into(),
        );

        let wc = w.clone();
        w.ui.window().on_close_requested(move || {
            wc.quit();
            CloseRequestResponse::HideWindow
        });

        Ok(w)
    }

    pub async fn show(&self) -> Result<()> {
        self.ui.show()?;

        match self.app_ctx.client.authenticate().await {
            Ok(state) => match state {
                AuthState::Authenticated => {
                    self.ui.set_is_loading(false);
                    self.ui
                        .set_loading_status("Authenticated successfully.".into());
                }
                AuthState::NeedsAuthentication => {
                    self.ui.set_is_loading(true);
                    self.ui.set_loading_status(
                        "Please authenticate with your NextCloud instance in your browser..."
                            .into(),
                    );
                }
            },
            Err(e) => {
                self.ui.set_is_loading(false);
                self.ui
                    .set_loading_status(format!("Authentication failed: {}", e).into());
            }
        }

        Ok(())
    }

    pub fn quit(&self) {
        slint::quit_event_loop().unwrap();
    }
}

impl NCClientEvents for App {
    fn on_login(&self, result: OnLogin) {
        match result {
            OnLogin::Success => {
                self.ui.set_is_loading(false);
                self.ui
                    .set_loading_status("Authenticated successfully.".into());
            }
            OnLogin::Failure(e) => {
                self.ui.set_is_loading(false);
                self.ui
                    .set_loading_status(format!("Authentication failed: {}", e).into());
            }
        }
    }
}
