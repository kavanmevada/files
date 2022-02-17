use gtk::{self, gio, glib};

use gio::prelude::FileEnumeratorExt;
use gio::prelude::FileExt;
use gio::prelude::FileExtManual;
use glib::clone;

pub trait Utilities {
    fn iter<'a>(&'a self, store: &'a gio::ListStore);
}

impl Utilities for gio::File {
    fn iter(&self, store: &gio::ListStore) {
        self.enumerate_children_async(
            "standard::*",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            glib::PRIORITY_DEFAULT,
            None::<&gio::Cancellable>,
            clone!(@strong self as s, @strong store => move |result| {
                if let Ok(enumrator) = result {
                    while let Ok(Some(info)) = enumrator.next_file(None::<&gio::Cancellable>) {
                        if info.file_type() == gio::FileType::Directory {
                            let subdir = s.resolve_relative_path (info.name());
                            subdir.iter(&store);
                        } else {
                            store.append(&info);
                        }
                    }
                }
            }),
        );
    }
}
