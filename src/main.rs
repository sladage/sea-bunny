use crate::{
    app_context::AppContext,
    models::config::{Config, config},
    services::ncclient::NCClient,
};

mod app_context;
mod dto;
mod models;
mod services;
mod ui;

fn main() -> anyhow::Result<()> {
    Config::init();

    if config().nextcloud_server.is_none() {
        ui::start_ui_first_run(&run_app)?;
    } else {
        drop(slint::spawn_local(async {
            run_app().await.unwrap();
        }));
    }

    slint::run_event_loop_until_quit()?;
    Ok(())
}

async fn run_app() -> anyhow::Result<()> {
    let ctx = AppContext {
        client: NCClient::new(config().nextcloud_server.unwrap())
            .await
            .expect("Unable to create NCClient."),
    };
    ui::start_ui(ctx)?;
    Ok(())
}
