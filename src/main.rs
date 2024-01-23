use std::time::SystemTime;

use adw::prelude::*;
use gtk::{gio, glib};

mod settings;
mod state;
mod widgets;
mod window;

const APP_ID: &str = "local.app.Pomodoro";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum State {
    Working { until: SystemTime },
    Pause { until: SystemTime },
}

impl Default for State {
    fn default() -> Self {
        State::Pause {
            until: SystemTime::now(),
        }
    }
}

fn main() -> glib::ExitCode {
    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);
    glib::g_info!("Pomodoro", "App: {APP_ID}");
    glib::g_info!("Pomodoro", "Version: {VERSION}");
    app.connect_activate(start);
    app.run()
}

fn start(app: &adw::Application) {
    gio::resources_register_include!("resources.gresource").unwrap();

    let window = window::Window::new(app);
    window.present();
}
