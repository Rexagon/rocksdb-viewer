use gtk::glib::ExitCode;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::ui;
use crate::window;

pub struct App {
    app: gtk::Application,
}

impl App {
    pub fn new() -> Self {
        let res = Self {
            app: gtk::Application::new(Some(crate::APP_ID), gio::ApplicationFlags::HANDLES_OPEN),
        };

        res.new_action("about", {
            let about = ui::about_dialog(None::<&gtk::Window>);
            move |_, _| Self::on_about(&about)
        });

        res.app.connect_startup(|_| {});
        res.app.connect_activate(Self::on_activate);
        res.app.connect_open(Self::on_open);

        res
    }

    fn on_about(about: &gtk::AboutDialog) {
        if about.window().is_some() {
            about.present();
        } else {
            about.run();
        }
    }

    fn on_activate(app: &gtk::Application) {
        let window = window::Window::new(app, None);
        app.add_window(window.as_ref());
    }

    fn on_open(app: &gtk::Application, files: &[gio::File], _hint: &str) {
        for file in files {
            if let Some(path) = file.path() {
                let window = window::Window::new(app, Some(path));
                app.add_window(window.as_ref());
                break;
            }
        }
    }

    pub fn run(&self) -> ExitCode {
        self.app.run()
    }

    pub fn new_action<F: Fn(&gio::SimpleAction, Option<&glib::Variant>) + 'static>(
        &self,
        name: &str,
        f: F,
    ) {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(f);
        self.add_action(&action);
    }
}

impl AsRef<gtk::Application> for App {
    #[inline]
    fn as_ref(&self) -> &gtk::Application {
        &self.app
    }
}

impl ActionMapExt for App {
    fn add_action(&self, action: &impl IsA<gio::Action>) {
        self.app.add_action(action);
    }
    fn lookup_action(&self, action_name: &str) -> Option<gio::Action> {
        self.app.lookup_action(action_name)
    }
    fn remove_action(&self, action_name: &str) {
        self.app.remove_action(action_name);
    }
}
