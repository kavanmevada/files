use glib::{clone, subclass::prelude::*};
use gtk::prelude::*;
use gtk::{self, gio, glib};

use crate::progress_info_model::ProgressInfoModel;

pub enum ProcessType {
    Copy,
    Move,
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &"org.kavanmevada.Files"),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .expect("Failed to create Application")
    }

    pub fn do_sync(&self, _action: ProcessType, dest: &gio::File) {
        let store = &self.imp().0 .0;

        let model: ProgressInfoModel = glib::Object::new(&[(
            "msg",
            &format!(
                "Downloading 3 items to {}",
                dest.basename().unwrap().to_str().unwrap()
            ),
        )])
        .expect("Failed to create ProgressInfoModel");

        let sender = model.imp().sender.clone();

        // dbg!(self.imp().store.borrow().n_items());

        let total_size = store.snapshot().iter().fold(0i64, |mut sum, val| {
            val.downcast_ref::<gio::FileInfo>().map_or(sum, |v| {
                sum += v.size();
                sum
            })
        });

        for item in store
            .snapshot()
            .iter()
            .filter_map(glib::Object::downcast_ref::<gio::FileInfo>)
        {
            let name = item.display_name();
            let dest = dest.child(&*name);

            if let Some(src) = item
                .attribute_object("standard::file")
                .and_then(|o| o.downcast::<gio::File>().ok())
            {
                src.copy_async(
                    &dest,
                    gio::FileCopyFlags::OVERWRITE,
                    glib::PRIORITY_LOW,
                    Some(&model.imp().cancellable),
                    Some(Box::from(clone!(@strong sender, @strong total_size => move |x: i64, _| {
                        //sender.send(Ok::<i64, glib::Error>(x)).expect("Error sending value");
                        sender.send(x as f64 / total_size as f64).expect("Error sending value");
                    }))),
                    clone!(@strong sender => move |ret| {
                        sender.send(if ret.is_ok() { 1.0 } else { -1.0 }).expect("Error sending value");
                    }),
                );
            }
        }

        self.imp().0 .1.append(&model);

        dbg!(self.imp().0 .1.n_items());
    }
}

mod imp {
    use glib::clone;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{self, gio, glib};

    use crate::browser_view::BrowserView;
    use crate::window::Window;

    #[derive(Debug, Default)]
    // By implementing Default we don't have to provide a `new` fn in our ObjectSubclass impl.
    pub struct Application(pub (gio::ListStore, gio::ListStore));

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self, obj: &Self::Type) {
            self.0
                 .1
                .connect_items_changed(clone!(@strong obj => move |_, _, _, _| {
                    obj.notify("n-process-str");
                    obj.notify("has-process");
                }));
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "selected-items-store",
                        "selected-items-store",
                        "selected-items-store",
                        gio::ListStore::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "processes-store",
                        "processes-store",
                        "processes-store",
                        gio::ListStore::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "n-process-str",
                        "n-process-str",
                        "n-process-str",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "has-process",
                        "has-process",
                        "has-process",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-items-store" => self.0 .0.to_value(),
                "processes-store" => self.0 .1.to_value(),
                "n-process-str" => self.0 .1.n_items().to_string().to_value(),
                "has-process" => (self.0 .1.n_items() > 0).to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self, application: &Self::Type) {
            let window = Window::new(Some(application));
            window.add_page(&BrowserView::for_path("/bin"), "/bin");
            window.add_page(&BrowserView::for_path("/lib64"), "/lib64");
            window.add_page(
                &BrowserView::for_path(glib::home_dir()),
                glib::home_dir().as_path().to_str().unwrap(),
            );
            window.add_css_class("devel");
            window.present();
        }

        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);

            let provider = gtk::CssProvider::new();
            provider.load_from_data(include_bytes!("../stylesheet.css"));
            gtk::StyleContext::add_provider_for_display(
                &gtk::gdk::Display::default().expect("Error initializing gtk css provider."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            // Set icons for shell
            gtk::Window::set_default_icon_name("org.kavanmevada.Files");

            let action_about = gio::SimpleAction::new("about", None);
            action_about.connect_activate(clone!(@strong self as s, @strong app => move |_, _| {
                let dialog = gtk::AboutDialog::builder()
                    .logo_icon_name("org.kavanmevada.Files")
                    .license_type(gtk::License::MitX11)
                    .website("https://example.com/files")
                    .version("0.0.1")
                    .transient_for(&app.active_window().unwrap())
                    .translator_credits("translator-credits")
                    .modal(true)
                    .authors(vec!["Application Developer".into()])
                    .artists(vec!["Application Developer".into()])
                    .build();

                dialog.present();
            }));

            let action_quit = gio::SimpleAction::new("quit", None);
            action_quit.connect_activate(clone!(@strong self as s, @strong app => move |_, _| {
                app.quit();
            }));

            app.add_action(&action_quit);
            app.add_action(&action_about);

            app.set_accels_for_action("app.quit", &["<Control>q"]);
        }
    }

    impl GtkApplicationImpl for Application {}
}
