mod app;
mod controller;
mod ui;
mod window;

pub const APP_NAME: &str = "RocksDB Viewer";
pub const APP_ID: &str = "com.broxus.RocksdbViewer";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> gtk::glib::ExitCode {
    gtk::init().expect("Failed to start GTK. Please install GTK3.");
    app::App::new().run()
}
