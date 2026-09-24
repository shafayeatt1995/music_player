#![allow(dead_code)]

mod app;
mod audio;
mod lyrics;
mod playlist;
mod ui;
mod utils;
mod video;

use app::XorinApp;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Xorin Player")
            .with_inner_size([980.0, 660.0])
            .with_min_inner_size([760.0, 500.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Xorin Player",
        native_options,
        Box::new(|cc| Ok(Box::new(XorinApp::new(cc)))),
    )
}
