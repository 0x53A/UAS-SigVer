#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
#[cfg(target_arch = "wasm32")]
mod font_wasm;
mod fonts;

use egui::{FontData, FontDefinitions, FontFamily};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl eframe::App for app::AliasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
    }
}

// for some reason we need an empty main for android, the actual entry point is in lib.rs
#[cfg(target_os = "android")]
fn main() {}

// When compiling natively:
#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "UAS SigVer: Aliasing Demonstration",
        native_options,
        Box::new(|cc| {
            add_fonts_to_ctx(&cc.egui_ctx);
            Ok(Box::new(app::AliasApp::default()))
        }),
    )
}
