use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk::{Application, ApplicationWindow};

use crate::controller;
use crate::ui;

pub struct Window {
    window: ApplicationWindow,
    view_stack: gtk::Stack,
    welcome_page_view: WelcomPageView,
    db_page_view: Rc<DbPageView>,
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let menu_bar = ui::MenuBar::new();

        let window = ApplicationWindow::new(app);
        window.set_default_size(1280, 720);
        window.set_position(gtk::WindowPosition::Center);
        window.set_type_hint(gdk::WindowTypeHint::Dialog);
        window.set_title(crate::APP_NAME);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        main_box.pack_start(menu_bar.as_ref(), false, false, 0);

        let folder_dialog = ui::folder_dialog(&window);

        let welcome_page_view = WelcomPageView::new();
        let db_page_view = Rc::new(DbPageView::new());

        let view_stack = gtk::Stack::new();
        view_stack.set_expand(true);
        view_stack.add(&welcome_page_view.container);
        view_stack.add(&db_page_view.container);

        main_box.add(&view_stack);
        window.add(&main_box);

        // Connect signals
        menu_bar
            .open
            .connect_activate(glib::clone!(@strong folder_dialog => move |_| {
                folder_dialog.run();
            }));
        menu_bar
            .about
            .connect_activate(glib::clone!(@weak app => move |_| {
                app.activate_action("about", None);
            }));
        menu_bar
            .exit
            .connect_activate(glib::clone!(@strong window => move |_| {
                window.close();
            }));

        folder_dialog.connect_response({
            let window = window.clone();
            let view_stack = view_stack.clone();
            let db_page_view = db_page_view.clone();

            move |file_chooser, response| {
                if response == gtk::ResponseType::Ok {
                    println!("{:?}", file_chooser.filename());

                    if let Some(filename) = file_chooser.filename() {
                        match controller::Db::open(filename) {
                            Ok(db) => {
                                db_page_view.cf_list.update_cfs(db.column_families());
                                view_stack.set_visible_child(&db_page_view.container);
                            }
                            Err(e) => {
                                ui::error_dialog(&window, &format!("{e:?}")).show_all();
                            }
                        };
                    }
                }
                file_chooser.hide();
            }
        });

        welcome_page_view.open_btn.connect_clicked(
            glib::clone!(@strong folder_dialog => move |_| {
                folder_dialog.show_all();
            }),
        );

        window.show_all();

        Self {
            window,
            view_stack,
            welcome_page_view,
            db_page_view,
        }
    }
}

impl AsRef<ApplicationWindow> for Window {
    fn as_ref(&self) -> &ApplicationWindow {
        &self.window
    }
}

struct WelcomPageView {
    container: gtk::Box,
    open_btn: gtk::Button,
}

impl WelcomPageView {
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

struct DbPageView {
    container: gtk::Box,
    cf_list: ui::CfList,
    db: Option<controller::Db>,
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

        Self {
            container,
            cf_list,
            db: None,
        }
    }
}
