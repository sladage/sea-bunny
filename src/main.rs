use crate::models::config::Config;

mod app_context;
mod dto;
mod events;
mod models;
mod services;
mod ui;

fn main() {
    Config::init();
}
