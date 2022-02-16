use crate::utilities::Utilities;
use glib::*;
use glib::subclass::prelude::*;
use gtk::glib;
use gtk::*;

glib::wrapper! {
    pub struct BrowserView(ObjectSubclass<imp::BrowserView>) @extends gtk::Widget, @implements gtk::Buildable;
}

impl BrowserView {
    pub fn for_path<P: AsRef<std::path::Path>>(path: P) -> Self {
        glib::Object::new(&[("path", &gio::File::for_path(path))]).expect("Failed to create Window")
    }

    pub fn search(&self, query: String) {
        self.imp().filter_model.set_filter(Some(&gtk::CustomFilter::new(move |obj| {
            let info = obj
                .downcast_ref::<gio::FileInfo>()
                .expect("The object needs to be of type `IntegerObject`.");

            info.name().display().to_string().contains(&query)
        })));
    }

    pub fn attach_search_view(&self) {
        self.property::<gio::File>("path").iter(&self.imp().sstore);
        self.imp().sort_model.set_model(Some(&self.imp().sstore));
    }

    pub fn detach_search_view(&self) {
        self.imp().sstore.remove_all();
        self.imp().sort_model.set_model(Some(&self.imp().list.get()));
    }
}

mod imp {
    use glib::clone;
    use std::cell::RefCell;

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
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
        #[template_child(id = "filter-model")]
        pub filter_model: TemplateChild<gtk::FilterListModel>,

        pub view_type: RefCell<Option<String>>,
        pub search_model: gtk::FilterListModel,

        pub sstore: gio::ListStore,
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

        #[template_callback(function = false)]
        fn filebrowser_activate(&self, pos: u32, grid: &gtk::GridView) {
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
                    self.list.set_file(Some(&file));
                } else {
                    gio::AppInfo::default_for_type(mime_type.as_str(), true).map(|info| {
                        info.launch(&[file], None::<&gdk::AppLaunchContext>)
                            .expect("Error launching app");
                    });
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

            item.child().map(|c| c.add_controller(&drag));
            item.child().map(|c| c.add_controller(&gesture));

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
            Self {
                view: Default::default(),
                stack: Default::default(),
                viewport: Default::default(),
                list: Default::default(),
                model: Default::default(),
                sort_model: Default::default(),
                filter_model: Default::default(),
                view_type: Default::default(),
                search_model: Default::default(),
                sstore: gio::ListStore::new(gio::FileInfo::static_type()),
            }
        }
    }

    impl ObjectImpl for BrowserView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "stack",
                        "stack",
                        "stack",
                        gtk::Stack::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "path",
                        "path",
                        "path",
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
                ]
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
                "path" => {
                    if let Ok(value) = value.get::<gio::File>() {
                        self.list.set_file(Some(&value));
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "stack" => self.stack.get().to_value(),
                "path" => self.list.file().unwrap().to_value(),
                "model" => self.model.to_value(),
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
