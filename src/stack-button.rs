use gtk::{self, glib};

glib::wrapper! {
    pub struct AdwStackButton(ObjectSubclass<imp::AdwStackButton>) @extends gtk::Widget, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl Default for AdwStackButton {
    fn default() -> Self {
        Self::new()
    }
}

impl AdwStackButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AdwStackButton")
    }
}

mod imp {
    use gtk::{self, gio, glib};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::glib::clone;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Debug)]
    pub struct AdwStackButton {
        pub stack: Rc<RefCell<Option<gtk::SelectionModel>>>,
        pub menu_btn: gtk::MenuButton,
        pub popover: gtk::PopoverMenu,
        pub button: gtk::Button,

        pub pos: Rc<RefCell<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AdwStackButton {
        const NAME: &'static str = "AdwStackButton";
        type Type = super::AdwStackButton;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("splitbutton");
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn new() -> Self {
            Self {
                stack: Default::default(),
                menu_btn: gtk::MenuButton::new(),
                popover: gtk::PopoverMenu::from_model(None::<&gio::MenuModel>),
                button: gtk::Button::new(),
                pos: Default::default(),
            }
        }
    }

    impl ObjectImpl for AdwStackButton {
        fn constructed(&self, obj: &Self::Type) {
            if let Some(child) = self.menu_btn.first_child() {
                child.remove_css_class("image-button")
            }

            self.button.connect_clicked(clone!(@strong self.stack as stack, @strong self.pos as pos => move |_btn| {
                pos.replace_with(|&mut pos| if stack.borrow().as_ref().map(|s| pos+1 >= s.n_items()).unwrap_or(false) { 0 } else { pos + 1 });

                let curr = *pos.borrow();

                if let Some(model) = stack.borrow().as_ref() {
                    model.select_item(*pos.borrow(), true);
                    let next_pos = if curr+1 >= model.n_items() { 0 } else { curr + 1 };
                    if let Some(item) = model.item(next_pos) { _btn.set_icon_name(&item.property::<String>("icon-name")) }
                }
            }));

            self.button.set_parent(obj);
            self.menu_btn.set_popover(Some(&self.popover));
            self.menu_btn.set_parent(obj);
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
                        "menu-model",
                        "menu-model",
                        "menu-model",
                        gio::MenuModel::static_type(),
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
                "stack" => {
                    if let Ok(stack) = value.get::<gtk::Stack>() {
                        let pages = stack.pages();

                        let mut selected_pos = 0;
                        for pos in 0..pages.n_items() {
                            if pages.is_selected(pos) {
                                selected_pos = pos;
                            }
                        }

                        let next = if selected_pos + 1 >= stack.pages().n_items() {
                            0
                        } else {
                            selected_pos + 1
                        };
                        let icon = stack
                            .pages()
                            .item(next)
                            .map(|i| i.property::<String>("icon-name"));
                        self.button.set_property("icon-name", icon);
                        self.pos.replace(selected_pos);
                    }

                    self.stack
                        .replace(value.get::<gtk::Stack>().ok().map(|val| val.pages()));
                }
                "menu-model" => self
                    .popover
                    .set_menu_model(value.get::<gio::MenuModel>().ok().as_ref()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "stack" => self.stack.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl BuildableImpl for AdwStackButton {}
    impl WidgetImpl for AdwStackButton {
        fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(widget, width, height, baseline);
        }
    }
}
