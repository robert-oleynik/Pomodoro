use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use adw::prelude::*;
use directories::ProjectDirs;
use gtk::glib::ControlFlow;
use gtk::{gio, glib, SignalListItemFactory};
use notify_rust::Notification;
use rodio::Sink;
use widgets::timer::Timer;

mod state;
mod widgets;
mod window;

const APP_ID: &str = "local.app.Pomodoro";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const WORK_DUR: Duration = Duration::from_secs(25 * 60);
const PAUSE_DUR: Duration = Duration::from_secs(5 * 60);

pub enum State {
    Working { until: SystemTime },
    Pause { until: SystemTime },
}

fn main() -> glib::ExitCode {
    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);
    glib::g_info!("Pomodoro", "App: {APP_ID}");
    glib::g_info!("Pomodoro", "Version: {VERSION}");
    app.connect_activate(start);
    app.run()
}

fn read_tasks(model: &gio::ListStore) {
    let data_file = ProjectDirs::from("local", "app", "Pomodoro")
        .unwrap()
        .data_dir()
        .to_str()
        .unwrap()
        .to_string();
    let data_file = format!("{data_file}/tasks");
    let reader = match File::open(data_file) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            glib::g_warning!("Pomodoro.Tasks", "{err}");
            return;
        }
    };
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                glib::g_warning!("Pomodoro.Tasks", "{err}");
                return;
            }
        };
        let entry = state::todo::Entry::new(false, line);
        add_entry(model, &entry);
    }
}

fn add_entry(model: &gio::ListStore, entry: &state::todo::Entry) {
    entry.connect_done_notify(glib::clone!(@weak model => move |_| {
        save_tasks(&model)
    }));
    entry.connect_desc_notify(glib::clone!(@weak model => move |_| {
        save_tasks(&model)
    }));
    model.append(entry);
}

fn save_tasks(list: &gio::ListStore) {
    let data_dir = ProjectDirs::from("local", "app", "Pomodoro")
        .unwrap()
        .data_local_dir()
        .to_str()
        .unwrap()
        .to_string();
    if let Err(err) = std::fs::create_dir_all(&data_dir) {
        glib::g_warning!("Pomodoro", "{err}");
        return;
    }
    let data_file = format!("{data_dir}/tasks");
    let mut writer = match File::create(data_file).map(BufWriter::new) {
        Ok(writer) => writer,
        Err(err) => {
            glib::g_warning!("Pomodoro", "{err}");
            return;
        }
    };
    for i in 0..list.n_items() {
        let item = list
            .item(i)
            .unwrap()
            .downcast::<state::todo::Entry>()
            .unwrap();
        if item.done() {
            continue;
        }
        let desc = item.desc();
        if let Err(err) = writer.write(format!("{desc}\n").as_bytes()) {
            glib::g_warning!("Pomodoro", "{err}");
        }
    }
}

fn alert(state: &State) {
    let message = match state {
        State::Pause { .. } => "Pause Complete",
        State::Working { .. } => "Work Complete",
    };
    let handle = Notification::new()
        .summary("Pomodoro Round End")
        .body(message)
        .icon("gnome-pomodoro")
        .show()
        .unwrap();
    match rodio::OutputStream::try_default() {
        Ok((_stream, handle)) => {
            match rodio::Decoder::new(Cursor::new(include_bytes!("notify_end.wav"))) {
                Ok(audio) => {
                    if let Err(err) = Sink::try_new(&handle).map(|sink| {
                        sink.append(audio);
                        sink.sleep_until_end();
                    }) {
                        glib::g_error!("Pomodoro.Alert", "{err}");
                    }
                }
                Err(err) => {
                    glib::g_error!("Pomodoro.Alert", "{err}");
                }
            };
        }
        Err(err) => {
            glib::g_error!("Pomodoro.Alert", "{err}")
        }
    }
    gio::spawn_blocking(move || {
        let _ = handle;
    });
}

fn start(app: &adw::Application) {
    gio::resources_register_include!("resources.gresource").unwrap();

    let settings = gio::Settings::new(APP_ID);
    let mut round = 0;
    let duration_work = Duration::from_secs(settings.uint64("duration-work"));
    let duration_short_pause = Duration::from_secs(settings.uint64("duration-short-pause"));
    let duration_long_pause = Duration::from_secs(settings.uint64("duration-long-pause"));
    let long_pause_every_round = Duration::from_secs(settings.uint64("long-pause-every-round"));

    let state = Rc::new(RefCell::new(State::Pause {
        until: SystemTime::now(),
    }));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Pomodoro")
        .build();

    let notified = Rc::new(Cell::new(true));

    let todo_model = gio::ListStore::new::<state::todo::Entry>();
    read_tasks(&todo_model);
    todo_model.connect_items_changed(|this, _, _added, _removed| save_tasks(this));

    let todo_list_factory = SignalListItemFactory::new();
    todo_list_factory.connect_setup(|_, item| {
        let entry = widgets::todo::Entry::default();
        item.downcast_ref::<gtk::ListItem>()
            .unwrap()
            .set_child(Some(&entry));
    });
    todo_list_factory.connect_bind(|_, item| {
        let state = item
            .downcast_ref::<gtk::ListItem>()
            .unwrap()
            .item()
            .and_downcast::<state::todo::Entry>()
            .unwrap();
        let entry = item
            .downcast_ref::<gtk::ListItem>()
            .unwrap()
            .child()
            .and_downcast::<widgets::todo::Entry>()
            .unwrap();
        entry.bind(&state)
    });

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .build();

    let timer = Timer::default();
    let gstate = state.clone();
    let gtimer = timer.clone();
    let gnotified = notified.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        let now = SystemTime::now();
        match *gstate.as_ref().borrow() {
            State::Pause { until } | State::Working { until } if until < now => {
                if !gnotified.get() {
                    gnotified.set(true);
                    let state = gstate.borrow();
                    alert(&*state);
                }
                match now.duration_since(until) {
                    Ok(u) => gtimer.set_property("time_secs", -(u.as_secs() as i32)),
                    Err(err) => glib::g_error!("Pomodoro", "{err}"),
                }
            }
            State::Pause { until } | State::Working { until } => match until.duration_since(now) {
                Ok(u) => gtimer.set_property("time_secs", u.as_secs() as i32),
                Err(err) => glib::g_error!("Pomodoro", "{err}"),
            },
        };
        ControlFlow::Continue
    });
    let gstate = state.clone();
    let gnotified = notified.clone();
    timer.connect_next(move |timer| {
        gnotified.set(false);
        let now = SystemTime::now();
        let new = match *gstate.as_ref().borrow() {
            State::Pause { .. } => State::Working {
                until: match now.checked_add(WORK_DUR) {
                    Some(time) => time,
                    None => return,
                },
            },
            State::Working { .. } => State::Pause {
                until: match now.checked_add(PAUSE_DUR) {
                    Some(time) => time,
                    None => return,
                },
            },
        };
        let secs = match new {
            State::Pause { until } | State::Working { until } => match until.duration_since(now) {
                Ok(u) => u.as_secs(),
                Err(err) => {
                    glib::g_error!("Pomodoro", "{err}");
                    return;
                }
            },
        };
        gstate.replace(new);
        timer.set_property("time_secs", secs as i32);
    });
    container.append(&timer);

    let todo_entry = gtk::Entry::builder()
        .margin_top(10)
        .secondary_icon_name("list-add-symbolic")
        .build();
    todo_entry.connect_icon_press(glib::clone!(@weak todo_model => move |entry, _| {
        let text: String = entry.buffer().property("text");
        let new_todo = state::todo::Entry::new(false, text);
        todo_model.append(&new_todo);
        entry.buffer().set_text("");
        glib::g_debug!("Pomodoro", "add new todo: {new_todo:?}");
    }));
    todo_entry.connect_activate(glib::clone!(@weak todo_model => move |entry| {
        let text: String = entry.buffer().property("text");
        let new_todo = state::todo::Entry::new(false, text);
        todo_model.append(&new_todo);
        entry.buffer().set_text("");
        glib::g_debug!("Pomodoro", "add new todo: {new_todo:?}");
    }));
    container.append(&todo_entry);

    let selection_model = gtk::NoSelection::new(Some(todo_model));
    let todo_view = gtk::ListView::new(Some(selection_model), Some(todo_list_factory));
    let scroll_area = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .css_classes(vec!["boxed-list", "card"])
        .vexpand(true)
        .child(&todo_view)
        .build();
    container.append(&scroll_area);

    let content = adw::Clamp::builder().child(&container).build();
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new("Pomodoro", ""))
        .build();
    container.append(&header);
    container.append(&content);

    window.set_content(Some(&container));
    window.present()
}
