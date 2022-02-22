use std::path::Path;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{self, gdk, gio, glib};

use crate::application::ProcessType;
use crate::browser_view::BrowserView;
use crate::window;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, @implements gio::ActionMap, gio::ActionGroup;
}

async fn appchooser_dialog(window: Rc<window::Window>, content_type: Rc<glib::GString>) {
    let dialog = gtk::AppChooserDialog::for_content_type(
        Some(&*window),
        gtk::DialogFlags::MODAL
            | gtk::DialogFlags::DESTROY_WITH_PARENT
            | gtk::DialogFlags::USE_HEADER_BAR,
        &*content_type,
    );

    let answer = dialog.run_future().await;
    dialog.close();

    let _ctx = gdk::Display::app_launch_context(&window.display());
    if answer == gtk::ResponseType::Ok {
        if let Some(single) = gtk::SelectionFilterModel::new(Some(
            &window.property::<gtk::MultiSelection>("selection-model"),
        ))
        .item(0)
        .and_then(|item| item.downcast::<gio::FileInfo>().ok())
        .and_then(|info| info.attribute_object("standard::file"))
        .and_then(|o| o.downcast::<gio::File>().ok())
        .map(|f| f.uri())
        {
            dialog
                .app_info()
                .and_then(|info| {
                    info.launch_uris(&[&single], None::<&gdk::AppLaunchContext>)
                        .ok()
                })
                .expect("Error launching app!");
        }
    }
}

async fn dialog(window: Rc<window::Window>) {
    let boxx = gtk::Box::builder()
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .spacing(4)
        .orientation(gtk::Orientation::Vertical)
        .build();

    let label = gtk::Label::new(Some("Folder name"));
    label.set_xalign(0.0);
    boxx.append(&label);

    let entry = gtk::Entry::new();
    entry.set_hexpand(true);
    boxx.append(&entry);

    let question_dialog = gtk::Dialog::builder()
        .transient_for(&*window)
        .modal(true)
        .use_header_bar(1)
        .title("Create New Folder")
        .child(&boxx)
        .build();

    question_dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Create", gtk::ResponseType::Ok),
    ]);
    question_dialog.set_default_response(gtk::ResponseType::Ok);

    let answer = question_dialog.run_future().await;
    question_dialog.close();

    if answer == gtk::ResponseType::Ok {
        let cancellable = gio::Cancellable::new();
        let dir: gio::File = window.property::<BrowserView>("selected-page-child").property("path");
        dir.child(entry.text().as_str())
            .make_directory(Some(&cancellable))
            .expect("Error making directory");
    }
}

impl Window {
    pub fn create_tab<P: AsRef<Path>>(&self, path: P) {
        let child = BrowserView::for_path(path.as_ref());
        let page = self.imp().tabview.add_page(&child, None);
        if let Some(title) = path.as_ref().to_str() {
            page.set_title(title);
        }
    }

    pub fn new<P: glib::IsA<gtk::Application> + ToValue>(app: Option<&P>) -> Self {
        if let Some(app) = app {
            glib::Object::new(&[("application", &app)])
        } else {
            glib::Object::new(&[])
        }
        .expect("Failed to create Window")
    }
}

mod imp {

    use crate::process_item_view::ProcessItemView;

    use crate::application::Application;
    use crate::browser_view::BrowserView;
    use crate::progress_info_model::ProgressInfoModel;
    use crate::stack_button::AdwStackButton;

    use glib::clone;
    
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::{self, gdk, gio, glib};
    use std::cell::RefCell;

    use std::rc::Rc;

    #[derive(Debug, Default)]
    pub enum Selection {
        Single(gio::FileInfo),
        Multi(gtk::SelectionFilterModel),
        #[default]
        None,
    }

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "window.ui")]
    pub struct Window {
        #[template_child(id = "view-port")]
        pub view_port: TemplateChild<gtk::Box>,

        #[template_child(id = "tab-bar")]
        pub tabbar: TemplateChild<adw::TabView>,
        #[template_child(id = "tab-view")]
        pub tabview: TemplateChild<adw::TabView>,
        #[template_child]
        pub popover: TemplateChild<gtk::PopoverMenu>,

        pub menu_page: RefCell<Option<adw::TabPage>>,
        pub application: RefCell<Option<Application>>,
        pub selection_model: RefCell<Option<gtk::MultiSelection>>,

        pub path: RefCell<Option<gio::File>>,
        pub selected_view: RefCell<Option<BrowserView>>,
        pub single_selection: RefCell<Option<gio::FileInfo>>,

        pub selection: Rc<RefCell<Selection>>,

        pub view_type: RefCell<Option<String>>,


        #[template_child(id = "show-hidden-btn")]
        pub show_hidden_btn: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);

            klass.install_action("open-dir", None, |win, _name, _variant| {
                if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                    let selection_file = selection
                        .attribute_object("standard::file")
                        .and_then(|o| o.downcast::<gio::File>().ok());
                    win.property::<BrowserView>("selected-page-child")
                        .set_property("dir", selection_file.as_ref());
                }
            });

            klass.install_action("open-with-other", None, |win, _name, _variant| {
                if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                    if let Some(content_type) = selection.content_type() {
                        gtk::glib::MainContext::default().spawn_local(super::appchooser_dialog(
                            Rc::clone(&Rc::new(win.clone())),
                            Rc::clone(&Rc::new(content_type)),
                        ));
                    }
                }
            });

            klass.install_action("new-folder", None, |win, _name, _variant| {
                gtk::glib::MainContext::default()
                    .spawn_local(super::dialog(Rc::clone(&Rc::new(win.clone()))));
            });

            klass.install_action("open-in-new-window", None, |win, _name, _variant| {
                if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                    if let Some(path) = selection
                        .attribute_object("standard::file")
                        .and_then(|o| o.downcast::<gio::File>().ok())
                        .and_then(|f| f.path())
                    {
                        if selection.file_type() == gio::FileType::Directory {
                            let nwindow = super::Window::new(win.application().as_ref());
                            nwindow
                                .create_tab(&path);
                            nwindow.present();
                        }
                    }
                }
            });

            klass.install_action("open-in-new-tab", None, |win, _name, _variant| {
                if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                    if let Some(path) = selection
                        .attribute_object("standard::file")
                        .and_then(|o| o.downcast::<gio::File>().ok())
                        .and_then(|f| f.path())
                    {
                        if selection.file_type() == gio::FileType::Directory {
                            win.create_tab(&path);
                        }
                    }
                }
            });

            klass.install_action("move-to-new-window", None, |win, _name, _variant| {
                let nwindow = super::Window::new(win.application().as_ref());
                if let Some(selected) = win.imp().menu_page.borrow().as_ref() {
                    win.imp()
                        .tabview
                        .transfer_page(&selected, &nwindow.imp().tabview.get(), 0);
                    nwindow.present();
                } else {
                    nwindow.destroy();
                }
            });

            klass.install_action("paste", None, |win, _name, _variant| {
                win.property::<Application>("application").do_sync(
                    super::ProcessType::Copy,
                    &win.property("selected-view-path"),
                );
            });

            klass.install_action("copy", None, |win, _name, _variant| {
                if let Selection::Multi(selection) = &*win.imp().selection.borrow() {
                    let app: Application = win.property("application");
                    let store: gio::ListStore = app.property("selected-items-store");

                    store.remove_all();
                    for item in selection.snapshot() {
                        store.append(&item);
                    }
                }
            });

            klass.install_action("open-in-default", None, |win, _name, _variant| {
                if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                    if let (Some(mime_type), Some(file)) = (
                        selection.content_type(),
                        selection
                            .attribute_object("standard::file")
                            .and_then(|f| f.downcast::<gio::File>().ok()),
                    ) {
                        if let Some(info) = gio::AppInfo::default_for_type(mime_type.as_str(), true)
                        {
                            info.launch(&[file], None::<&gdk::AppLaunchContext>)
                                .expect("Error launching in default app!");
                        }
                    }
                }
            });

            klass.install_action("open-in-terminal", None, |win, _name, _variant| {
                if let Some(selected_file) =
                    if let Selection::Single(selection) = &*win.imp().selection.borrow() {
                        selection.attribute_object("standard::file").and_then(|f| {
                            f.downcast::<gio::File>().ok().and_then(|f| {
                                if selection.file_type() == gio::FileType::Directory {
                                    Some(f)
                                } else {
                                    None
                                }
                            })
                        })
                    } else {
                        None
                    }
                    .or(win.property::<BrowserView>("selected-page-child").property("dir"))
                {
                    gio::AppInfo::create_from_commandline(
                        "gnome-terminal --working-directory",
                        None,
                        gio::AppInfoCreateFlags::NONE,
                    )
                    .and_then(|cmdline| {
                        cmdline.launch(&[selected_file], None::<&gio::AppLaunchContext>)
                    })
                    .expect("Error launching commandline");
                }
            });

            klass.install_action("sort-by-name", None, |win, _name, _variant| {
                let view = win.property::<BrowserView>("selected-page-child");
                view.imp()
                    .sort_model
                    .set_sorter(Some(&gtk::CustomSorter::new(move |obj1, obj2| {
                        let app_info1 = obj1.downcast_ref::<gio::FileInfo>().unwrap();
                        let app_info2 = obj2.downcast_ref::<gio::FileInfo>().unwrap();

                        app_info1
                            .name()
                            .into_os_string()
                            .to_ascii_lowercase()
                            .cmp(&app_info2.name().into_os_string().to_ascii_lowercase())
                            .into()
                    })))
            });

            klass.install_action("sort-by-modified", None, |win, _name, _variant| {
                let view = win.property::<BrowserView>("selected-view");
                view.imp()
                    .sort_model
                    .set_sorter(Some(&gtk::CustomSorter::new(move |obj1, obj2| {
                        let app_info1 = obj1.downcast_ref::<gio::FileInfo>().unwrap();
                        let app_info2 = obj2.downcast_ref::<gio::FileInfo>().unwrap();

                        app_info1
                            .modification_date_time()
                            .cmp(&app_info2.modification_date_time())
                            .into()
                    })))
            });

            // klass.install_action("show-hidden", None, |win, _name, _variant| {
            //     let _view = win.property::<BrowserView>("selected-view");
            //     //view.imp().filter_model.set_filter(None::<&gtk::Filter>);
            // });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            AdwStackButton::static_type();
            BrowserView::static_type();
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl Window {
        #[template_callback(function = false)]
        fn toggle_hiddent_state(&self, btn: gtk::CheckButton) {
            self.selected_view.borrow().as_ref().map(|v| v.set_property("show-hidden", btn.is_active()));
        }

        #[template_callback(function = false)]
        fn go_forward(&self) {
            self.selected_view.borrow().as_ref().map(|v| v.go_forward());
        }

        #[template_callback(function = false)]
        fn go_backward(&self) {
            self.selected_view.borrow().as_ref().map(|v| v.go_backward());
        }

        // #[template_callback(function = false)]
        // fn search_started(&self, entry: &gtk::SearchEntry) {
        //     dbg!(self.selected_view.borrow().as_ref());
        //     self.selected_view.borrow().as_ref().map(|v| v.attach_search_view(entry));
        // }

        // #[template_callback(function = false)]
        // fn search_stopped(&self) {
        //     dbg!(self.selected_view.borrow().as_ref());
        //     self.selected_view.borrow().as_ref().map(|v| v.detach_search_view());
        // }

        #[template_callback(function = false)]
        fn search_entry_changed(&self, entry: &gtk::SearchEntry) {
            dbg!("ASDasdasdasd");
            if let Some(view) = self.selected_view.borrow().as_ref() {
                if entry.text() != "" {
                    view.search(entry.text().to_string());
                } else {
                    view.detach_search_view();
                }
            }
        }

        #[template_callback(function = false)]
        fn switch_view_cb(&self, _view: &adw::TabView) -> adw::TabView {
            let window = super::Window::new(self.application.borrow().as_ref());
            window.present();
            window.imp().tabview.get()
        }

        #[template_callback(function = false)]
        fn create_window_cb(&self, _view: &adw::TabView) -> adw::TabView {
            let window = super::Window::new(self.application.borrow().as_ref());
            window.present();
            window.imp().tabview.get()
        }

        #[template_callback(function = false)]
        fn setup_menu_cb(&self, page: Option<adw::TabPage>, _view: &adw::TabView) {
            self.menu_page.replace(page);
        }

        #[template_callback]
        fn factory_setup(&self, item: &gtk::ListItem, _factory: &gtk::ListItemFactory) {
            let view = ProcessItemView::new();

            let item_expr = gtk::PropertyExpression::new(
                gtk::ListItem::static_type(),
                Some(&gtk::ConstantExpression::new(item)),
                "item",
            );

            gtk::PropertyExpression::new(ProgressInfoModel::static_type(), Some(&item_expr), "msg")
                .bind(&view.imp().msg.get(), "label", None::<&glib::Object>);

            gtk::PropertyExpression::new(
                ProgressInfoModel::static_type(),
                Some(&item_expr),
                "fraction",
            )
            .bind(
                &view.imp().progressbar.get(),
                "fraction",
                None::<&glib::Object>,
            );

            if let Some(application) = self.application.borrow().as_ref() {
                let store = application.imp().0 .1.clone();

                view.imp()
                    .cancel_btn
                    .connect_clicked(clone!(@strong item => move |_| {
                        store.remove(item.position());
                    }));
            }

            item.set_child(Some(&view));
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            let gesture = gtk::GestureClick::new();
            self.view_port.add_controller(&gesture);

            gesture.set_button(3);
            gesture.connect_released(clone!(@strong obj, @strong self.popover as popover, @strong self.selection as selection => move |gesture, _n_press, x, y| {
                let application = obj.application().and_then(|app| app.downcast::<Application>().ok()).unwrap();



                let selected_model: gio::ListStore = application.property("selected-items-store");
                let selection_model = obj.property::<gtk::MultiSelection>("selection-model");
                let filter_model = gtk::SelectionFilterModel::new(Some(&selection_model));

                if let Some(found) = gesture.widget().pick(x, y, gtk::PickFlags::all()) {
                    if found.type_() == gtk::GridView::static_type() || found.type_() == gtk::ListView::static_type() {
                        selection_model.unselect_all();
                    }
                }

                let single = filter_model
                        .item(0)
                        .and_then(|item| item.downcast::<gio::FileInfo>().ok());

                selection.replace(if filter_model.n_items() > 1 {
                    Selection::Multi(filter_model)
                } else if let Some(single) = single.clone() {
                    Selection::Single(single)
                } else {
                    Selection::None
                });

                let is_single_dir = single.as_ref().map(|s| s.file_type() == gio::FileType::Directory).unwrap_or(false);

                let menu = gio::Menu::new();
                let section0 = gio::Menu::new();
                let section1 = gio::Menu::new();
                let section2 = gio::Menu::new();
                let section3 = gio::Menu::new();

                if is_single_dir {
                    section0.append(Some("Open"), Some("open-dir"));
                    section1.append(Some("Open in New Tab"), Some("open-in-new-tab"));
                    section1.append(Some("Open in New Window"), Some("open-in-new-window"));
                    section1.append(Some("Open in Terminal"), Some("open-in-terminal"));
                } else {

                    if let Some(info) = single.as_ref().and_then(|f| f.content_type())
                        .and_then(|mime| gio::AppInfo::default_for_type(mime.as_str(), true)) { section0.append(Some(&format!("Open with {}", info.name().as_str())), Some("open-in-default")) }


                    section3.append(Some("New Folder"), Some("new-folder"));
                    section1.append(Some("Open Terminal Here"), Some("open-in-terminal"));
                }

                if single.is_some() {
                    section2.append(Some("Copy"), Some("copy"));
                }

                if selected_model.n_items() > 0 {
                    section2.append(Some("Paste"), Some("paste"));
                }

                section0.append(Some("Open with Other Application"), Some("open-with-other"));



                menu.append_section(None, &section0);
                menu.append_section(None, &section1);
                menu.append_section(None, &section2);
                menu.append_section(None, &section3);
                popover.set_menu_model(Some(&menu));


                popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                popover.popup();
            }));

            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "selection-model",
                        "selection-model",
                        "selection-model",
                        gtk::MultiSelection::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "selected-page-child",
                        "selected-page-child",
                        "selected-page-child",
                        BrowserView::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "selection-model" => {
                    if let Ok(model) = value.get() {
                        self.selection_model.replace(Some(model));
                    }
                }
                "selected-page-child" => {
                    if let Ok(view) = value.get::<BrowserView>() {
                        self.selected_view.replace(Some(view));
                    }
                }

                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selection-model" => self.selection_model.borrow().to_value(),
                "selected-page-child" => self.selected_view.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
}
