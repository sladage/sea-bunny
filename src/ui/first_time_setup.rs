use super::UiShard;
use crate::{models::config::config, ui::generated};
use anyhow::Result;
use eventful_rs::*;
use slint::{CloseRequestResponse, ComponentHandle, SharedString};
use url::Url;

use_shard!(shard = UiShard);

#[slint_events(component = generated::FirstTimeSetupWindow)]
trait UiEvents {
    fn connect(&self, server: SharedString);
}

#[events]
pub trait FirstTimeSetupEvents {
    fn on_first_time_setup_complete(&self);
}

#[eventful(FirstTimeSetupEvents)]
pub struct FirstTimeSetup {
    ui: generated::FirstTimeSetupWindow,
    ui_events: UiEventsBridge,
}

impl FirstTimeSetup {
    pub fn new() -> Result<ShardRc<Self>> {
        let ui = generated::FirstTimeSetupWindow::new()?;
        let w = Self::bind_local(Self {
            ui_events: UiEventsBridge::new(&ui),
            ui,
            events: Default::default(),
        })?;

        w.ui_events.connect_to(&w);

        w.ui.window().on_close_requested(|| {
            slint::quit_event_loop().unwrap();
            CloseRequestResponse::HideWindow
        });

        Ok(w)
    }

    pub fn show(&self) -> Result<()> {
        self.ui.show()?;
        Ok(())
    }
}

impl UiEvents for FirstTimeSetup {
    fn connect(&self, server: SharedString) {
        if server.is_empty() {
            self.ui.set_has_error(true);
            self.ui.set_message("Server URL cannot be empty.".into());
            self.ui.set_input_enabled(true);
            return;
        }

        println!("Connecting to server: {}", server);
        match Url::parse(server.as_str()) {
            Ok(server) => {
                config().update(|cfg| cfg.nextcloud_server = Some(server));
                self.events.on_first_time_setup_complete().emit();
                self.ui.hide().unwrap();
            }
            Err(e) => {
                self.ui.set_has_error(true);
                self.ui.set_message(format!("Invalid URL: {}", e).into());
                self.ui.set_input_enabled(true);
            }
        }
    }
}
