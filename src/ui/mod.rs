use crate::{app_context::AppContext, events::Event};

slint::include_modules!();

pub fn start_ui(ctx: AppContext) {
    let a = AppWindow::new().unwrap();
    a.show();
}

pub fn start_ui_first_run(on_done_setup: Event<()>) {
    let a = AppWindow::new().unwrap();
    a.show();
}
