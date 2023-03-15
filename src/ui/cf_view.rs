use gtk::glib;
use gtk::prelude::*;

pub struct CfView {
    pub tree_view: gtk::TreeView,
    pub store: gtk::ListStore,
}

impl CfView {
    pub fn new() -> Self {
        const COLUMN_TYPES: [glib::Type; 2] = [glib::Type::STRING, glib::Type::STRING];

        let store = gtk::ListStore::new(&COLUMN_TYPES);
        let tree_view = gtk::TreeView::with_model(&store);
        tree_view.set_vexpand(true);

        add_text_column(&tree_view, Column::Key, "Key");
        add_text_column(&tree_view, Column::Value, "Value");

        Self { tree_view, store }
    }

    pub fn update<I>(&self, iter: I)
    where
        I: Iterator<Item = (String, String)>,
    {
        self.store.clear();
        for (key, value) in iter {
            self.store.set(
                &self.store.append(),
                &[(Column::Key as u32, &key), (Column::Value as u32, &value)],
            );
        }
    }
}

impl AsRef<gtk::TreeView> for CfView {
    fn as_ref(&self) -> &gtk::TreeView {
        &self.tree_view
    }
}

fn add_text_column(tree_view: &gtk::TreeView, column: Column, title: &str) {
    let renderer = gtk::CellRendererText::new();
    let view_column = gtk::TreeViewColumn::new();
    view_column.set_resizable(true);
    view_column.set_title(title);
    TreeViewColumnExt::pack_start(&view_column, &renderer, true);
    TreeViewColumnExt::add_attribute(&view_column, &renderer, "text", column as i32);
    tree_view.append_column(&view_column);
}

#[derive(Debug)]
#[repr(i32)]
enum Column {
    Key,
    Value,
}
