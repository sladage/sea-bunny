slint::include_modules!();

pub fn run_ui() {
    let a = AppWindow::new().unwrap();
    a.show();
    slint::run_event_loop();
}
