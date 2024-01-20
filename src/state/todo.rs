use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::Entry)]
    pub struct Entry {
        #[property(get, set)]
        done: Cell<bool>,
        #[property(get, set)]
        desc: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Entry {
        const NAME: &'static str = "TodoListEntry";
        type Type = super::Entry;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Entry {}
}

glib::wrapper! {
    pub struct Entry(ObjectSubclass<imp::Entry>);
}

impl Entry {
    pub fn new(done: bool, desc: impl Into<String>) -> Self {
        glib::Object::builder()
            .property("done", done)
            .property("desc", desc.into())
            .build()
    }
}
