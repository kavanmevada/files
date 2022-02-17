use gtk::{self, glib};

glib::wrapper! {
    pub struct ProgressInfoModel(ObjectSubclass<imp::ProgressInfoModel>);
}

mod imp {
    use glib::*;
    use gtk::subclass::prelude::*;
    use gtk::{self, gio, glib};
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct ProgressInfoModel {
        pub msg: RefCell<Option<String>>,
        pub sender: glib::Sender<f64>,
        pub receiver: RefCell<Option<glib::Receiver<f64>>>,
        pub fraction: RefCell<f64>,
        pub source: RefCell<Option<glib::Source>>,
        pub context: glib::MainContext,
        pub cancellable: gio::Cancellable,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProgressInfoModel {
        const NAME: &'static str = "ProgressInfoModel";
        type Type = super::ProgressInfoModel;

        fn new() -> Self {
            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            Self {
                msg: Default::default(),
                sender,
                receiver: RefCell::new(Some(receiver)),
                fraction: RefCell::from(0.0),
                source: Default::default(),
                context: glib::MainContext::default(),
                cancellable: gio::Cancellable::new(),
            }
        }
    }

    impl ObjectImpl for ProgressInfoModel {
        fn constructed(&self, obj: &Self::Type) {
            let receiver = self
                .receiver
                .borrow_mut()
                .take()
                .expect("Error getting receiver");
            let fraction = &self.fraction;

            let sourceid = receiver.attach(
                Some(&self.context),
                clone!(@strong fraction, @strong obj => move |f| {
                    obj.set_property("fraction", f);
                    glib::Continue(true)
                }),
            );

            self.source
                .replace(self.context.find_source_by_id(&sourceid));
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::new(
                        "fraction",
                        "fraction",
                        "fraction",
                        f64::MIN,
                        f64::MAX,
                        0.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "msg",
                        "msg",
                        "msg",
                        None,
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
                "fraction" => {
                    if let Ok(value) = value.get() {
                        self.fraction.replace(value);
                    }
                }
                "msg" => {
                    if let Ok(value) = value.get() {
                        self.msg.replace(value);
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "fraction" => self.fraction.borrow().to_value(),
                "msg" => self.msg.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(source) = self.source.borrow().as_ref() {
                source.destroy()
            }
        }
    }
}
