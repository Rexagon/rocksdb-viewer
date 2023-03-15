use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use gtk::{gdk, gio, glib, prelude::*};
use gtk::{Application, ApplicationWindow};

use crate::controller;
use crate::ui;

pub struct Window(Rc<WindowState>);

impl Window {
    pub fn new(app: &Application, initial_path: Option<PathBuf>) -> Self {
        let menu_bar = ui::MenuBar::new();

        let window = ApplicationWindow::new(app);
        window.set_default_size(1280, 720);
        window.set_position(gtk::WindowPosition::Center);
        window.set_type_hint(gdk::WindowTypeHint::Dialog);
        window.set_title(crate::APP_NAME);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.pack_start(menu_bar.as_ref(), false, false, 0);

        let welcome_page_view = WelcomePageView::new();
        let db_page_view = DbPageView::new();

        let view_stack = gtk::Stack::new();
        view_stack.set_expand(true);
        view_stack.add(&welcome_page_view.container);
        view_stack.add(&db_page_view.container);

        main_box.add(&view_stack);
        window.add(&main_box);

        let folder_dialog = ui::folder_dialog(&window);

        let shared_state = Rc::new(WindowState {
            window,
            db: Default::default(),
            view_stack,
            welcome_page_view,
            db_page_view,
        });

        // Connect signals

        menu_bar
            .open
            .connect_activate(glib::clone!(@weak folder_dialog => move |_| {
                folder_dialog.run();
            }));
        menu_bar
            .about
            .connect_activate(glib::clone!(@weak app => move |_| {
                app.activate_action("about", None);
            }));
        menu_bar.exit.connect_activate(
            glib::clone!(@weak shared_state.window as window => move |_| {
                window.close();
            }),
        );

        folder_dialog.connect_response(
            glib::clone!(@strong shared_state => move |file_chooser, response| {
                if response == gtk::ResponseType::Ok {
                    if let Some(path) = file_chooser.filename() {
                        shared_state.open_db(path)
                    }
                }
                file_chooser.hide();
            }),
        );

        shared_state.welcome_page_view.open_btn.connect_clicked(
            glib::clone!(@strong folder_dialog => move |_| {
                folder_dialog.show_all();
            }),
        );

        shared_state.window.show_all();

        if let Some(path) = initial_path {
            shared_state.open_db(path);
        }
        Self(shared_state)
    }
}

impl AsRef<ApplicationWindow> for Window {
    fn as_ref(&self) -> &ApplicationWindow {
        &self.0.window
    }
}

struct WindowState {
    window: ApplicationWindow,
    db: RefCell<Option<controller::Db>>,
    view_stack: gtk::Stack,
    welcome_page_view: WelcomePageView,
    db_page_view: DbPageView,
}

impl WindowState {
    fn open_db(&self, path: PathBuf) {
        let opened_db = match controller::Db::open(path) {
            Ok(db) => db,
            Err(e) => {
                ui::error_dialog(&self.window, &format!("{e:?}")).show_all();
                return;
            }
        };

        let mut db = self.db.borrow_mut();
        let db = db.insert(opened_db);

        self.db_page_view.cf_list.update_cfs(db.column_families());
        self.select_page(&self.db_page_view);
    }

    fn select_page<T: AsRef<gtk::Box>>(&self, page: &T) {
        self.view_stack.set_visible_child(page.as_ref());
    }
}

struct WelcomePageView {
    container: gtk::Box,
    open_btn: gtk::Button,
}

impl WelcomePageView {
    fn new() -> Self {
        let open_btn = gtk::Button::new();
        open_btn.set_label("Open RocksDB folder");
        open_btn.set_halign(gtk::Align::Center);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.pack_start(&open_btn, true, false, 0);

        Self {
            container,
            open_btn,
        }
    }
}

impl AsRef<gtk::Box> for WelcomePageView {
    fn as_ref(&self) -> &gtk::Box {
        &self.container
    }
}

struct DbPageView {
    container: gtk::Box,
    cf_list: ui::CfList,
}

impl DbPageView {
    fn new() -> Self {
        let cf_list = ui::CfList::new();

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.add1(&cf_list.tree_view);

        let main_view = gtk::Box::new(gtk::Orientation::Vertical, 0);
        paned.add2(&main_view);

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.pack_start(&paned, true, true, 0);

        Self { container, cf_list }
    }
}

impl AsRef<gtk::Box> for DbPageView {
    fn as_ref(&self) -> &gtk::Box {
        &self.container
    }
}
