use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::*;
use gtk::glib;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/local/app/Pomodoro/settings.ui")]
    pub struct Settings {
        #[template_child]
        round: gtk::TemplateChild<adw::SpinRow>,
        #[template_child]
        mins_long_break: gtk::TemplateChild<adw::SpinRow>,
        #[template_child]
        mins_short_break: gtk::TemplateChild<adw::SpinRow>,
        #[template_child]
        mins_work: gtk::TemplateChild<adw::SpinRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Settings {
        const NAME: &'static str = "PomodoroSettings";
        type Type = super::Settings;
        type ParentType = adw::PreferencesWindow;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl PreferencesWindowImpl for Settings {}
    impl AdwWindowImpl for Settings {}
    impl WindowImpl for Settings {}
    impl WidgetImpl for Settings {}

    impl ObjectImpl for Settings {
        fn constructed(&self) {
            self.parent_constructed();

            let settings = self.obj().settings();
            settings
                .bind_property("duration-work", &*self.mins_work, "value")
                .transform_to(|_, number: i32| Some((number / 60).to_value()))
                .transform_from(|_, number: i32| Some((number * 60).to_value()))
                .bidirectional()
                .sync_create()
                .build();
            settings
                .bind_property("duration-short-pause", &*self.mins_work, "value")
                .transform_to(|_, number: i32| Some((number / 60).to_value()))
                .transform_from(|_, number: i32| Some((number * 60).to_value()))
                .bidirectional()
                .sync_create()
                .build();
            settings
                .bind_property("duration-long-pause", &*self.mins_work, "value")
                .transform_to(|_, number: i32| Some((number / 60).to_value()))
                .transform_from(|_, number: i32| Some((number * 60).to_value()))
                .bidirectional()
                .sync_create()
                .build();
            settings
                .bind_property("long-pause-every-round", &*self.round, "value")
                .bidirectional()
                .sync_create()
                .build();
        }
    }
}

glib::wrapper! {
    pub struct Settings(ObjectSubclass<imp::Settings>)
        @extends adw::PreferencesWindow, adw::Window, gtk::Window, gtk::Widget;
}

impl Settings {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
