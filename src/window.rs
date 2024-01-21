use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use adw::prelude::*;
use adw::subclass::prelude::*;
use directories::ProjectDirs;
use glib::subclass::*;
use gtk::{gio, glib};

use crate::{state, widgets};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties, Default)]
    #[template(resource = "/local/app/Pomodoro/window.ui")]
    #[properties(wrapper_type = super::Window)]
    pub struct Window {
        #[template_child]
        todo_entry: gtk::TemplateChild<gtk::Entry>,
        #[template_child]
        todo_list: gtk::TemplateChild<gtk::ListView>,
        #[template_child]
        todo_factory: gtk::TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        timer: gtk::TemplateChild<widgets::Timer>,
        #[property(get, set)]
        work_secs: Rc<RefCell<u64>>,
        #[property(get, set)]
        short_pause_secs: Rc<RefCell<u64>>,
        #[property(get, set)]
        long_pause_secs: Rc<RefCell<u64>>,
        #[property(get, set)]
        long_pause_every_round: Rc<RefCell<u64>>,
        // State
        state: Rc<RefCell<state::State>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "PomodoroApplication";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let settings = gio::Settings::new(crate::APP_ID);
            self.work_secs.replace(settings.uint64("duration-work"));
            self.short_pause_secs
                .replace(settings.uint64("duration-short-pause"));
            self.long_pause_secs
                .replace(settings.uint64("duration-long-pause"));
            self.long_pause_every_round
                .replace(settings.uint64("long-pause-every-round"));

            self.todo_factory.connect_setup(|_, item| {
                let entry = crate::widgets::todo::Entry::default();
                item.downcast_ref::<gtk::ListItem>()
                    .unwrap()
                    .set_child(Some(&entry))
            });
            self.todo_factory.connect_bind(|_, item| {
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

            let todo_model = self
                .todo_list
                .model()
                .and_downcast::<gtk::NoSelection>()
                .unwrap()
                .model()
                .and_downcast::<gio::ListStore>()
                .unwrap();
            read_tasks(&todo_model);
            self.todo_entry
                .connect_icon_press(glib::clone!(@weak todo_model => move |entry, _| {
                    let text: String = entry.buffer().property("text");
                    if Self::add_new_entry(&todo_model, text).is_none() {
                        glib::g_warning!("Pomdoro", "failed to add new entry");
                    }
                    entry.buffer().set_text("");
                }));
            self.todo_entry
                .connect_activate(glib::clone!(@weak todo_model => move |entry| {
                    let text: String = entry.buffer().property("text");
                    if Self::add_new_entry(&todo_model, text).is_none() {
                        glib::g_warning!("Pomdoro", "failed to add new entry");
                    }
                    entry.buffer().set_text("");
                }));

            let state = self.state.clone();
            let timer = self.timer.clone();
            glib::timeout_add_local(Duration::from_secs(1), move || {
                let now = SystemTime::now();
                let mut state = state.as_ref().borrow_mut();
                if state.until < now {
                    if !state.notified {
                        alert(&state);
                        state.notified = true;
                    }
                    match now.duration_since(state.until) {
                        Ok(u) => timer.set_property("time_secs", -(u.as_secs() as i32)),
                        Err(err) => glib::g_warning!("Pomodoro", "{err}"),
                    }
                } else {
                    match state.until.duration_since(now) {
                        Ok(u) => timer.set_property("time_secs", u.as_secs() as i32),
                        Err(err) => glib::g_warning!("Pomodoro", "{err}"),
                    }
                }
                glib::ControlFlow::Continue
            });
            let state = self.state.clone();
            let work_secs = self.work_secs.clone();
            let short_pause_secs = self.short_pause_secs.clone();
            let long_pause_secs = self.long_pause_secs.clone();
            let long_pause_every_round = self.long_pause_every_round.clone();
            self.timer.connect_next(move |timer| {
                let mut state = state.as_ref().borrow_mut();
                let secs = match (state.state, state.round) {
                    (state::Pomodoro::Working, round)
                        if round % *long_pause_every_round.as_ref().borrow() == 0 =>
                    {
                        *long_pause_secs.as_ref().borrow()
                    }
                    (state::Pomodoro::Working, _) => *short_pause_secs.as_ref().borrow(),
                    _ => *work_secs.as_ref().borrow(),
                };
                timer.set_property("time_secs", secs);
                state.next(Duration::from_secs(secs));
            });
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}

    impl Window {
        pub fn add_new_entry(model: &gio::ListStore, text: impl Into<String>) -> Option<()> {
            let entry = state::todo::Entry::new(false, text);
            entry.connect_done_notify(glib::clone!(@weak model => move |_| {
                if let Err(err) = super::save_tasks(&model) {
                    glib::g_warning!("Pomodoro", "{err}");
                }
            }));
            entry.connect_desc_notify(glib::clone!(@weak model => move |_| {
                if let Err(err) = super::save_tasks(&model) {
                    glib::g_warning!("Pomodoro", "{err}");
                }
            }));
            model.append(&entry);
            if let Err(err) = save_tasks(&model) {
                glib::g_warning!("Pomodoro", "{err}");
            }
            glib::g_debug!("Pomodoro", "add new todo: {entry:?}");
            Some(())
        }
    }
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}

pub fn save_tasks(model: &gio::ListStore) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = ProjectDirs::from("local", "app", "Pomodoro")
        .unwrap()
        .data_local_dir()
        .to_str()
        .unwrap()
        .to_string();
    std::fs::create_dir_all(&data_dir)?;
    let data_file = format!("{data_dir}/tasks");
    let mut writer = File::create(data_file).map(BufWriter::new)?;
    for i in 0..model.n_items() {
        let item = model
            .item(i)
            .unwrap()
            .downcast::<state::todo::Entry>()
            .unwrap();
        if item.done() {
            continue;
        }
        let desc = item.desc();
        writer.write(desc.as_bytes())?;
        writer.write("\n".as_bytes())?;
    }
    Ok(())
}

pub fn alert(state: &state::State) {
    let message = match state.state {
        state::Pomodoro::Pause { .. } => format!("Round {}: Pause ended", state.round),
        state::Pomodoro::Working { .. } => format!("Round {}: Work ended", state.round),
    };
    let handle = notify_rust::Notification::new()
        .summary("Pomodoro")
        .body(&message)
        .icon("gnome-pomodoro")
        .show()
        .unwrap();
    match rodio::OutputStream::try_default() {
        Ok((_stream, handle)) => {
            match rodio::Decoder::new(Cursor::new(include_bytes!("notify_end.wav"))) {
                Ok(audio) => {
                    if let Err(err) = rodio::Sink::try_new(&handle).map(|sink| {
                        sink.append(audio);
                        sink.sleep_until_end();
                    }) {
                        glib::g_warning!("Pomodoro.Alert", "{err}");
                    }
                }
                Err(err) => {
                    glib::g_warning!("Pomodoro.Alert", "{err}");
                }
            };
        }
        Err(err) => {
            glib::g_warning!("Pomodoro.Alert", "{err}")
        }
    }
    gio::spawn_blocking(move || {
        let _ = handle;
    });
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
        imp::Window::add_new_entry(model, line);
    }
}
