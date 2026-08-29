use crate::{
    app_context::AppContext,
    events::EventSource,
    models::config::{Config, config},
    services::ncclient::NCClient,
};

mod app_context;
mod dto;
mod events;
mod models;
mod services;
mod ui;

fn main() -> anyhow::Result<()> {
    Config::init();

    if config().nextcloud_server.is_none() {
        let on_done_setup: EventSource<()> = EventSource::new();

        on_done_setup.listen(|_| {
            run_app();
        });

        ui::start_ui_first_run(on_done_setup.event());
    } else {
        run_app();
    }

    slint::run_event_loop()?;
    Ok(())
}

fn run_app() {
    let ctx = AppContext {
        client: NCClient::new(config().nextcloud_server.unwrap()),
    };
    ui::start_ui(ctx);
}
