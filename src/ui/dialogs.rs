use gtk::prelude::*;
use gtk::{gdk, glib};

pub fn about_dialog<'a, P: IsA<gtk::Window> + 'a, Q: Into<Option<&'a P>>>(
    parent: Q,
) -> gtk::AboutDialog {
    let title = format!("About {}", crate::APP_NAME);

    let p = gtk::AboutDialog::new();
    p.set_title(&title);
    p.set_program_name(crate::APP_NAME);
    p.set_authors(&["Broxus"]);
    p.set_license_type(gtk::License::Apache20);
    p.set_version(Some(crate::VERSION));
    p.set_website(Some("https://github.com/broxus/rocksdb-viewer"));
    p.set_website_label(Some("GitHub Repo"));

    p.set_destroy_with_parent(true);
    p.set_skip_pager_hint(true);
    p.set_skip_taskbar_hint(true);
    p.set_transient_for(parent.into());
    p.set_type_hint(gdk::WindowTypeHint::Splashscreen);

    p.connect_delete_event(|p, _| p.hide_on_delete());
    p.connect_response(|p, _| p.hide());

    if p.header_bar().is_none() {
        let hbar = gtk::HeaderBar::new();
        hbar.set_title(p.title().as_ref().map(glib::GString::as_str));
        hbar.show_all();
        p.set_titlebar(Some(&hbar));
    }

    p
}

pub fn folder_dialog<'a, P: IsA<gtk::Window> + 'a, Q: Into<Option<&'a P>>>(
    parent: Q,
) -> gtk::FileChooserDialog {
    let p = gtk::FileChooserDialog::new(
        Some("Open RocksDB folder"),
        parent.into(),
        gtk::FileChooserAction::SelectFolder,
    );

    p.set_destroy_with_parent(true);
    p.set_skip_pager_hint(true);
    p.set_skip_taskbar_hint(true);
    p.set_type_hint(gdk::WindowTypeHint::Splashscreen);
    p.connect_delete_event(|p, _| p.hide_on_delete());

    p.add_buttons(&[
        ("Open", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    p
}

pub fn error_dialog<'a, P: IsA<gtk::Window> + 'a, Q: Into<Option<&'a P>>>(
    parent: Q,
    msg: &impl std::fmt::Display,
) -> gtk::MessageDialog {
    let msg = msg.to_string();

    let p = gtk::MessageDialog::new(
        parent.into(),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        &msg,
    );

    p.connect_response(|dialog, _| dialog.close());
    p
}
