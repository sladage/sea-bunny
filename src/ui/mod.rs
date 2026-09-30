use crate::{
    app_context::AppContext,
    ui::{
        app_window::App,
        first_time_setup::{FirstTimeSetup, FirstTimeSetupEventsSignalsExt},
    },
};
use eventful_rs::*;

pub mod app_window;
pub mod first_time_setup;

declare_shard!(pub UiShard, runtime = slint);
use_shard!(shard = UiShard);

pub mod generated {
    slint::include_modules!();
}

pub fn start_ui(ctx: AppContext) -> anyhow::Result<()> {
    let a = App::new(ctx)?;
    a.show()?;

    Ok(())
}

pub fn start_ui_first_run<F>(run_app: &'static F) -> anyhow::Result<()>
where
    F: AsyncFn() -> anyhow::Result<()> + Sync + Send + 'static,
{
    let a = FirstTimeSetup::new()?;

    a.on_first_time_setup_complete()
        .on_shard(&UiShard::handle())
        .connect(|| {
            drop(slint::spawn_local(async {
                run_app().await.unwrap();
            }));
        });

    a.show()?;

    Ok(())
}
