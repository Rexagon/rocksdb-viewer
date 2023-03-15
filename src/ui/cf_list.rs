use gtk::prelude::*;
use gtk::{gio, glib};

pub struct CfList {
    pub tree_view: gtk::TreeView,
    pub store: gtk::ListStore,
}

impl CfList {
    pub fn new() -> Self {
        const COLUMN_TYPES: [glib::Type; 1] = [glib::Type::STRING];

        let store = gtk::ListStore::new(&COLUMN_TYPES);
        let tree_view = gtk::TreeView::with_model(&store);
        tree_view.set_vexpand(true);
        tree_view.set_search_column(Column::Name as i32);

        {
            let renderer = gtk::CellRendererText::new();
            let column = gtk::TreeViewColumn::new();
            TreeViewColumnExt::pack_start(&column, &renderer, true);
            column.set_title("Column Family");
            TreeViewColumnExt::add_attribute(&column, &renderer, "text", Column::Name as i32);
            tree_view.append_column(&column);
        }

        Self { tree_view, store }
    }

    pub fn update_cfs(&self, cfs: &[String]) {
        self.store.clear();
        for cf in cfs {
            self.store
                .set(&self.store.append(), &[(Column::Name as u32, cf)]);
        }
    }

    pub fn connect_cf_selected<F>(&self, f: F)
    where
        F: Fn(&str) + 'static,
    {
        let store = self.store.clone();
        self.tree_view.connect_row_activated(move |_, path, _| {
            let Some(iter) = store.iter(path) else {
                    return;
                };
            let value = store.value(&iter, Column::Name as i32);
            let value = value.get::<&str>().unwrap();
            f(value);
        });
    }
}

impl AsRef<gtk::TreeView> for CfList {
    #[inline]
    fn as_ref(&self) -> &gtk::TreeView {
        &self.tree_view
    }
}

#[derive(Debug)]
#[repr(i32)]
enum Column {
    Name,
}
