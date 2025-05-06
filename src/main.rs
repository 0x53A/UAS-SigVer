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
            add_font_to_ctx(cc, crate::fonts::UBUNTU_LIGHT.to_vec());
            Ok(Box::new(app::AliasApp::default()))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let decompressed_font = crate::font_wasm::decompress_gzip(crate::fonts::UBUNTU_LIGHT_GZIP)
            .await
            .expect("Failed to decompress font");
        // let decompressed_font = vec![];

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    add_font_to_ctx(cc, decompressed_font);
                    Ok(Box::new(app::AliasApp::default()))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn add_font_to_ctx(cc: &eframe::CreationContext<'_>, font_raw_ubuntu: Vec<u8>) {
    let mut fonts = FontDefinitions::default();

    // fonts.font_data.insert(
    //     "Hack".to_owned(),
    //     std::sync::Arc::new(
    //         // .ttf and .otf supported
    //         FontData::from_static(crate::fonts::HACK_REGULAR),
    //     ),
    // );

    fonts.font_data.insert(
        "Ubuntu-Light".to_owned(),
        std::sync::Arc::new(
            // .ttf and .otf supported
            FontData::from_owned(font_raw_ubuntu),
        ),
    );

    // fonts
    //     .families
    //     .get_mut(&FontFamily::Proportional)
    //     .unwrap()
    //     .insert(0, "Hack".to_owned());

    // fonts
    //     .families
    //     .get_mut(&FontFamily::Monospace)
    //     .unwrap()
    //     .push("Hack".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Ubuntu-Light".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("Ubuntu-Light".to_owned());

    let egui_ctx = &cc.egui_ctx;
    egui_ctx.set_fonts(fonts);
}
