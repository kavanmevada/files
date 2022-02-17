use gtk::{self, glib};

glib::wrapper! {
    pub struct ProcessItemView(ObjectSubclass<imp::ProcessItemView>) @extends gtk::Widget;
}

impl Default for ProcessItemView {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessItemView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ProcessItemView")
    }
}

mod imp {
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::{self, glib};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "process-item-view.ui")]
    pub struct ProcessItemView {
        #[template_child]
        pub progressbar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub msg: TemplateChild<gtk::Label>,
        #[template_child(id = "cancel-btn")]
        pub cancel_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessItemView {
        const NAME: &'static str = "ProcessItemView";
        type Type = super::ProcessItemView;
        type ParentType = gtk::Widget;
        // type Interfaces = (gtk::Orientable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProcessItemView {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ProcessItemView {}
}
