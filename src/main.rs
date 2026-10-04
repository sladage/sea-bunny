use std::sync::Arc;

use eventful_rs::ShardRc;

use crate::{
    app_context::AppContext,
    models::config::{Config, config},
    services::{ncclient::NCClient, notifications::NotificationServices, talk::TalkServices},
    ui::first_time_setup::FirstTimeSetup,
};

mod app_context;
mod dto;
mod models;
mod services;
mod ui;

fn main() -> anyhow::Result<()> {
    Config::init();
    let mut _first_time_setup_window: Option<ShardRc<FirstTimeSetup>> = None;

    if config().nextcloud_server.is_none() {
        _first_time_setup_window = Some(ui::start_ui_first_run(&run_app)?);
    } else {
        drop(slint::spawn_local(async {
            run_app().await.unwrap();
        }));
    }

    slint::run_event_loop_until_quit()?;
    Ok(())
}

async fn run_app() -> anyhow::Result<()> {
    let client = NCClient::new(config().nextcloud_server.unwrap())
        .await
        .expect("Unable to create NCClient.");
    let talk = TalkServices::new(&client)
        .await
        .expect("Unable to create Talk services.");
    let notifications = NotificationServices::new(&client)
        .await
        .expect("Unable to create notification services.");
    let ctx = Arc::new(AppContext {
        client,
        talk,
        notifications,
    });
    ui::start_ui(ctx).await?;
    Ok(())
}
