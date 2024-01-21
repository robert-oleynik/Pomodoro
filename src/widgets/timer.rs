use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use std::cell::Cell;

mod imp {
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties, Default)]
    #[template(resource = "/local/app/Pomodoro/widgets/timer.ui")]
    #[properties(wrapper_type = super::Timer)]
    pub struct Timer {
        #[property(get, set)]
        time_secs: Cell<i32>,
        #[template_child]
        timer: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub btn: gtk::TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Timer {
        const NAME: &'static str = "PomodoroTimer";
        type Type = super::Timer;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl BoxImpl for Timer {}
    impl WidgetImpl for Timer {}

    #[glib::derived_properties]
    impl ObjectImpl for Timer {
        fn constructed(&self) {
            self.parent_constructed();

            let timer: &gtk::Label = &*self.timer;
            let mins = self.time_secs.get() / 60;
            let secs = self.time_secs.get() % 60;
            let label = format!(r#"<span size="64pt">{mins}:{secs:0>2}</span>"#);
            timer.set_label(&label);
            self.obj()
                .connect_time_secs_notify(glib::clone!(@weak timer => move |obj| {
                    let secs_until: i32 = obj.property("time_secs");
                    let ch = (secs_until < 0).then(|| '-').unwrap_or(' ');
                    let (mins, secs) = match secs_until {
                        secs if secs < 0 => (-secs / 60, -secs % 60),
                        secs => (secs / 60, secs % 60),
                    };
                    let label = format!(r#"<span size="64pt">{ch}{mins}:{secs:0>2}</span>"#);
                    timer.set_label(&label);
                }));
            let obj = self.obj();
            self.btn
                .connect_clicked(glib::clone!(@weak obj => move |_| {
                    obj.emit_by_name::<()>("next", &[]);
                }));
        }
    }
}

glib::wrapper! {
    pub struct Timer(ObjectSubclass<imp::Timer>)
        @extends gtk::Box, gtk::Widget;
}

impl Default for Timer {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl Timer {
    pub fn connect_next(&self, f: impl Fn(&Timer) + 'static) {
        let this = self.imp();
        this.btn
            .connect_clicked(glib::clone!(@weak this => move |_| {
                let obj = this.obj();
                f(&*obj);
            }));
    }
}
