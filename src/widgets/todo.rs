use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties, Default)]
    #[template(resource = "/local/app/Pomodoro/widgets/todo.ui")]
    #[properties(wrapper_type = super::Entry)]
    pub struct Entry {
        #[property(get, set)]
        done: Cell<bool>,
        #[property(get, set)]
        desc: RefCell<String>,
        #[template_child]
        edit: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        entry: gtk::TemplateChild<gtk::Entry>,
        #[template_child]
        text: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        cdone: gtk::TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Entry {
        const NAME: &'static str = "TodoEntryWidget";
        type Type = super::Entry;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl BoxImpl for Entry {}
    impl WidgetImpl for Entry {}

    #[glib::derived_properties]
    impl ObjectImpl for Entry {
        fn constructed(&self) {
            self.parent_constructed();

            let this = self.obj();
            this.bind_property("done", &*self.cdone, "active")
                .bidirectional()
                .sync_create()
                .build();
            this.bind_property("desc", &*self.text, "label")
                .bidirectional()
                .sync_create()
                .build();
            this.bind_property("desc", &self.entry.buffer(), "text")
                .bidirectional()
                .sync_create()
                .build();

            let this = self.obj();
            let stack = &*self.stack;
            let entry = &*self.entry;
            let text = &*self.text;
            self.edit.connect_clicked(
                glib::clone!(@weak this, @weak stack, @weak entry, @weak text => move |_| {
                    let visible = stack.visible_child();
                    if let Some(visible) = visible {
                        if visible == entry {
                            stack.set_visible_child(&text);
                            return;
                        }
                    }
                    stack.set_visible_child(&entry);
                    entry.grab_focus();
                    entry.set_width_request(this.width() * 4 / 7);
                }),
            );
            self.entry.set_hexpand(true);
            self.entry.set_hexpand_set(true);
            self.entry
                .connect_activate(glib::clone!(@weak stack, @weak text => move |_| {
                    stack.set_visible_child(&text)
                }));
        }
    }
}

glib::wrapper! {
    pub struct Entry(ObjectSubclass<imp::Entry>)
        @extends gtk::Box, gtk::Widget;
}

impl Default for Entry {
    fn default() -> Self {
        glib::Object::builder()
            .property("done", false)
            .property("desc", String::from(""))
            .build()
    }
}

impl Entry {
    pub fn bind(&self, entry: &crate::state::todo::Entry) {
        let done: bool = entry.property("done");
        let desc: String = entry.property("desc");
        glib::g_debug!("Pomodoro.Todo", "entry {done:?} {desc:?}");
        entry
            .bind_property("done", self, "done")
            .bidirectional()
            .sync_create()
            .build();
        entry
            .bind_property("desc", self, "desc")
            .bidirectional()
            .sync_create()
            .build();
    }
}
