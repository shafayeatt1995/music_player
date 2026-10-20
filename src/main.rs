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

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../logo.ico");
    if let Ok(img) = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico) {
        let rgba = img.into_rgba8();
        let (width, height) = rgba.dimensions();
        return Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        });
    }
    None
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Xorin Player")
        .with_inner_size([980.0, 660.0])
        .with_min_inner_size([760.0, 500.0])
        .with_drag_and_drop(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Xorin Player",
        native_options,
        Box::new(|cc| Ok(Box::new(XorinApp::new(cc)))),
    )
}
