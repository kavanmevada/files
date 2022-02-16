use gtk::glib;
// use gtk::prelude::*;
// use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct ProcessPopover(ObjectSubclass<imp::ProcessPopover>) @extends gtk::Widget, gtk::Popover, @implements gtk::Buildable;
}

mod imp {

    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct ProcessPopover;

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessPopover {
        const NAME: &'static str = "ProcessPopover";
        type Type = super::ProcessPopover;
        type ParentType = gtk::Popover;
        type Interfaces = (gtk::Buildable,);
    }

    impl ObjectImpl for ProcessPopover {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        // fn properties() -> &'static [glib::ParamSpec] {
        //     use once_cell::sync::Lazy;
        //     static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        //         vec![glib::ParamSpecObject::new(
        //             "child",
        //             "child",
        //             "child",
        //             gtk::Widget::static_type(),
        //             glib::ParamFlags::READWRITE,
        //         )]
        //     });
        //     PROPERTIES.as_ref()
        // }

        // fn set_property(
        //     &self,
        //     obj: &Self::Type,
        //     _id: usize,
        //     value: &glib::Value,
        //     pspec: &glib::ParamSpec,
        // ) {
        //     match pspec.name() {
        //         "child" => if let Ok(child) = value.get::<gtk::Widget>() {
        //             child.set_parent(obj);
        //         }
        //         _ => unimplemented!(),
        //     }
        // }

        // fn dispose(&self, buildable: &Self::Type) {
        //     while let Some(child) = buildable.first_child() {
        //         child.unparent();
        //     }
        // }
    }

    impl WidgetImpl for ProcessPopover {}
    impl PopoverImpl for ProcessPopover {}
    impl BuildableImpl for ProcessPopover {}
}
