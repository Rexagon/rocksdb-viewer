use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

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
        tree_view.set_search_column(Columns::Name as i32);

        {
            let renderer = gtk::CellRendererText::new();
            let column = gtk::TreeViewColumn::new();
            TreeViewColumnExt::pack_start(&column, &renderer, true);
            column.set_title("Column Family");
            TreeViewColumnExt::add_attribute(&column, &renderer, "text", Columns::Name as i32);
            tree_view.append_column(&column);
        }

        Self { tree_view, store }
    }

    pub fn update_cfs(&self, cfs: &[String]) {
        self.store.clear();
        for cf in cfs {
            self.store
                .set(&self.store.append(), &[(Columns::Name as u32, cf)]);
        }
    }
}

#[derive(Debug)]
#[repr(i32)]
enum Columns {
    Name,
}
