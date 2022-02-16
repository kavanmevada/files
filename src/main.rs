#![feature(derive_default_enum)]

extern crate adw;
extern crate gtk;

mod utilities;
mod application;
mod window;

#[path = "browser-view.rs"]
mod browser_view;

#[path = "process-popover.rs"]
mod process_popover;

#[path = "stack-button.rs"]
mod stack_button;

#[path = "progress-info-model.rs"]
mod progress_info_model;

#[path = "process-item-view.rs"]
mod process_item_view;

use application::Application;
use gtk::prelude::*;

fn main() {
    gtk::init().expect("Error initializing GTK");
    adw::init();

    let app = Application::new();
    std::process::exit(app.run());
}
