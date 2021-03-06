use crate::utilities::Utilities;
use glib::subclass::prelude::*;

use gtk::{self, gio, glib::{self, clone}, prelude::*};

glib::wrapper! {
    pub struct BrowserView(ObjectSubclass<imp::BrowserView>) @extends gtk::Widget, @implements gtk::Buildable;
}

impl BrowserView {
    pub fn for_path<P: AsRef<std::path::Path>>(path: P) -> Self {
        glib::Object::new(&[("dir", &gio::File::for_path(path))]).expect("Failed to create Window")
    }

    pub fn search(&self, _query: String) {
        // self.imp().sfilter.set_filter(Some(&gtk::CustomFilter::new(clone!(@strong self as s => move |obj| {
        //     let info = obj
        //         .downcast_ref::<gio::FileInfo>()
        //         .expect("The object needs to be of type `IntegerObject`.");
        //     let str = info.name().display().to_string();

        //     str.contains(&query)
        // }))));
    }

    pub fn attach_search_view(&self, _entry: &gtk::SearchEntry) {
        self.imp().sort_model.set_model(Some(&self.imp().sstore));
        //self.property::<gio::File>("dir").iter(&self.imp().sstore);
    }

    pub fn detach_search_view(&self) {
        self.imp().sstore.remove_all();
        self.imp()
            .sort_model
            .set_model(Some(&self.imp().list.get()));
    }



    // Navigation Methods
    pub fn go_backward(&self) {
        let (store, pos) = &mut *self.imp().history.borrow_mut();
        if *pos > 0 {
            *pos = pos.saturating_sub(1);
            if let Some(item) = store.item(*pos)
                .and_then(|c| c.downcast::<gio::File>().ok()) {
                self.imp().list.get().set_file(Some(&item));
            }
        }
    }

    pub fn go_forward(&self) {
        let (store, pos) = &mut *self.imp().history.borrow_mut();
        if pos.saturating_add(1) < store.n_items() {
            *pos = pos.saturating_add(1);
            if let Some(item) = store.item(*pos)
                .and_then(|c| c.downcast::<gio::File>().ok()) {
                self.imp().list.get().set_file(Some(&item));
            }
        }
    }

    pub fn set_active(&self, state: bool) {
        self.imp().include_hidden.replace(state);
    }

    pub fn active(&self) -> bool {
        *self.imp().include_hidden.borrow()
    }

}

mod imp {
    use glib::clone;
    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    
    use std::collections::HashMap;
    use std::rc::Rc;

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{self, gdk, gio};
    use gtk::{glib, CompositeTemplate};

    #[derive(Debug, CompositeTemplate)]
    #[template(file = "browser-view.ui")]
    pub struct BrowserView {
        #[template_child]
        pub view: TemplateChild<gtk::GridView>,

        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub viewport: TemplateChild<gtk::Stack>,

        #[template_child]
        pub list: TemplateChild<gtk::DirectoryList>,

        #[template_child]
        pub model: TemplateChild<gtk::MultiSelection>,
        #[template_child(id = "sort-model")]
        pub sort_model: TemplateChild<gtk::SortListModel>,
        // #[template_child(id = "filter-model")]
        // pub filter_model: TemplateChild<gtk::FilterListModel>,

        pub view_type: RefCell<Option<String>>,
        pub search_model: gtk::FilterListModel,

        pub sstore: gio::ListStore,
        pub sfilter: gtk::FilterListModel,
        // pub filterMap: HashMap<String, gtk::CustomFilter>,

        pub history: Rc<RefCell<(gio::ListStore, u32)>>,

        pub include_hidden: Rc<RefCell<bool>>,

        #[template_child]
        pub filters: TemplateChild<gtk::EveryFilter>,

        pub hidden_filter: gtk::CustomFilter,
    }

    #[gtk::template_callbacks]
    impl BrowserView {
        #[template_callback(function = false)]
        fn filebrowser_loading_notify(&self) {
            self.viewport.set_visible_child_name(
                if !self.list.is_loading() && self.list.n_items() == 0 {
                    "is-empty"
                } else {
                    "not-empty"
                },
            );
        }

        #[template_callback]
        fn filebrowser_activate(view: &super::BrowserView, pos: u32, grid: &gtk::GridView) {
            let single = grid
                .model()
                .and_then(|g| g.item(pos))
                .and_then(|o| o.downcast::<gio::FileInfo>().ok());

            if let (Some(file_type), Some(mime_type), Some(file)) = (
                single.as_ref().map(|s| s.file_type()),
                single.as_ref().and_then(|s| s.content_type()),
                single
                    .and_then(|o| o.attribute_object("standard::file"))
                    .and_then(|f| f.downcast::<gio::File>().ok()),
            ) {
                if file_type == gio::FileType::Directory {
                    view.set_property("dir", file);
                } else if let Some(info) = gio::AppInfo::default_for_type(mime_type.as_str(), true)
                {
                    info.launch(&[file], None::<&gdk::AppLaunchContext>)
                        .expect("Error launching app");
                }
            }
        }

        #[template_callback(function = false)]
        fn filebrowser_get_display_name(item: &gtk::ListItem) -> Option<glib::GString> {
            item.item()
                .and_then(|item| item.downcast::<gio::FileInfo>().ok())
                .and_then(|o| o.attribute_string("standard::display-name"))
        }

        #[template_callback(function = false)]
        fn filebrowser_get_icon(item: &gtk::ListItem) -> Option<gio::Icon> {
            let gesture = gtk::GestureClick::new();
            gesture.set_button(0);
            gesture.connect_released(clone!(@strong item as item => move |gesture, _n_press, _x, _y| {
                if !item.is_selected() && gesture.current_button() == 3 {
                    item.child().map(|c| c.activate_action("listitem.select", Some(&(false, false).to_variant())));
                }
            }));

            // else if gesture.current_button() == 1 {
            //     let item = item.item().and_then(|item| item.downcast::<gio::FileInfo>().ok())
            //         .and_then(|info| info.attribute_object("standard::file"))
            //         .and_then(|item| item.downcast::<gio::File>().ok());
            //     dbg!(item);
            // }

            let drag = gtk::DragSource::new();
            drag.set_actions(gdk::DragAction::COPY);

            drag.connect_prepare(clone!(@strong item => move |_src, _x, _y| {
                let item = item.item().and_then(|item| item.downcast::<gio::FileInfo>().ok())
                    .and_then(|info| info.attribute_object("standard::file"))
                    .and_then(|item| item.downcast::<gio::File>().ok());
                // let canvas = src.widget();
                // let picked = canvas.pick(x, y, gtk::PickFlags::DEFAULT).expect("Error picking widget");
                // let a = picked.ancestor(gtk::ListItem::static_type());
                Some(gdk::ContentProvider::for_value(&item.to_value()))
            }));
            drag.connect_begin(clone!(@strong item => move |src, _event| {
                if let Some(child) = item.child() {
                    let paintable = gtk::WidgetPaintable::new(Some(&child));
                    src.set_icon(Some(&paintable), child.allocated_width(), child.allocated_height());
                    child.set_opacity(0.8);
                }
            }));
            drag.connect_end(clone!(@strong item => move |_src, _event| {
                let child = item.child();
                child.map(|c| c.set_opacity(1.0));
            }));

            if let Some(c) = item.child() {
                c.add_controller(&drag)
            }
            if let Some(c) = item.child() {
                c.add_controller(&gesture)
            }

            item.item()
                .and_then(|item| item.downcast::<gio::FileInfo>().ok())
                .and_then(|o| o.attribute_object("standard::icon"))
                .and_then(|icon| icon.downcast::<gio::Icon>().ok())
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BrowserView {
        const NAME: &'static str = "BrowserView";
        type Type = super::BrowserView;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {   
            let hidden_filter = gtk::CustomFilter::new(|obj| {
                let info = obj
                    .downcast_ref::<gio::FileInfo>()
                    .expect("The object needs to be of type `IntegerObject`.");
                let str = info.name().display().to_string();

                !str.starts_with(".")
            });

            let sstore = gio::ListStore::new(gio::FileInfo::static_type());
            let sfilter = gtk::FilterListModel::new(Some(&sstore), None::<&gtk::Filter>);

            Self {
                view: Default::default(),
                stack: Default::default(),
                viewport: Default::default(),
                list: Default::default(),
                model: Default::default(),
                sort_model: Default::default(),
                // filter_model: Default::default(),
                view_type: Default::default(),
                search_model: Default::default(),

                sstore,
                sfilter,

                history: Rc::new(RefCell::new((
                    gio::ListStore::new(gio::File::static_type()),
                    0u32,
                ))),

                include_hidden: Rc::new(RefCell::new(false)),

                filters: Default::default(),
                hidden_filter,
            }
        }
    }

    impl ObjectImpl for BrowserView {
        fn constructed(&self, obj: &Self::Type) {
            self.filters.append(&self.hidden_filter);
            self.parent_constructed(obj);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("go-backward", &[], <()>::static_type().into()).build(),
                    Signal::builder("go-forward", &[], <()>::static_type().into()).build()
                ]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "stack",
                    "stack",
                    "stack",
                    gtk::Stack::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecObject::new(
                    "dir",
                    "dir",
                    "dir",
                    gio::File::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecObject::new(
                    "model",
                    "model",
                    "model",
                    gio::File::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoolean::new(
                    "show-hidden",
                    "show-hidden",
                    "show-hidden",
                    false,
                    glib::ParamFlags::READWRITE,
                ),]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "show-hidden" => if let Ok(val) = value.get::<bool>() {
                    if !val {
                        self.filters.append(&self.hidden_filter);
                    } else {
                        // for pos in 0..self.filters.n_items() {
                        //     if let Some(name) = self.filters.item(pos).and_then(|i| unsafe { i.data::<String>("name") }) {
                        //         dbg!(unsafe { name.as_ref() });
                        //         if unsafe { name.as_ref() } == "hidden-filter" {
                        //             self.filters.remove(pos);
                        //         }
                        //     }
                        // }
                    }
                    self.include_hidden.replace(val);
                },

                "dir" => if let Ok(value) = value.get::<gio::File>() {
                    self.list.set_file(Some(&value));

                    let (store, pos) = &mut *self.history.borrow_mut();
                    for _ in (*pos).saturating_add(1)..store.n_items() { store.remove((*pos).saturating_add(1)) }
                    *pos = store.n_items();
                    store.append(&value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "stack" => self.stack.get().to_value(),
                "dir" => self.list.file().to_value(),
                "model" => self.model.to_value(),
                "show-hidden" => self.include_hidden.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for BrowserView {}
    impl BuildableImpl for BrowserView {}
}
