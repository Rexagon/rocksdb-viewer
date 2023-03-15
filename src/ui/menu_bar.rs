use gtk::prelude::*;

pub struct MenuBar {
    menu_bar: gtk::MenuBar,

    pub open: gtk::MenuItem,
    pub about: gtk::MenuItem,
    pub exit: gtk::MenuItem,
}

impl MenuBar {
    pub fn new() -> MenuBar {
        let menu_bar = gtk::MenuBar::new();

        let file = gtk::MenuItem::with_label("File");
        let file_menu = gtk::Menu::new();
        let file_menu_open = gtk::MenuItem::with_label("Open");
        let file_menu_about = gtk::MenuItem::with_label("About");
        let file_menu_exit = gtk::MenuItem::with_label("Exit");

        file_menu.add(&file_menu_open);
        file_menu.add(&gtk::SeparatorMenuItem::new());
        file_menu.add(&file_menu_about);
        file_menu.add(&gtk::SeparatorMenuItem::new());
        file_menu.add(&file_menu_exit);
        file.set_submenu(Some(&file_menu));
        menu_bar.add(&file);

        Self {
            menu_bar,
            open: file_menu_open,
            about: file_menu_about,
            exit: file_menu_exit,
        }
    }
}

impl AsRef<gtk::MenuBar> for MenuBar {
    fn as_ref(&self) -> &gtk::MenuBar {
        &self.menu_bar
    }
}
