use rust_web_component::WebComponent;
use rust_web_component_macro::WebComponent;
use wasm_bindgen::prelude::*;

use crate::app::AliasApp;
use crate::fonts::add_fonts_to_ctx;

// Implement eframe::App for AliasApp when building for wasm32
impl eframe::App for AliasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
    }
}

#[derive(WebComponent)]
#[web_component(name = "uas-sigver")]
pub struct UasSigverComponent {
    element: Option<web_sys::HtmlElement>,
    runner: Option<eframe::WebRunner>,
}

impl UasSigverComponent {
    fn new() -> Self {
        Self {
            element: None,
            runner: None,
        }
    }
}

impl WebComponent for UasSigverComponent {
    fn attach(&mut self, element: &web_sys::HtmlElement) {
        self.element = Some(element.clone());
    }

    fn connected(&mut self) {
        let element = self.element.as_ref().unwrap().clone();

        // Create shadow DOM
        let shadow = element
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .unwrap();

        // Create a canvas element inside the shadow DOM
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        // Set up styles for the shadow DOM
        shadow.set_inner_html(
            r#"
            <style>
                :host {
                    display: block;
                    width: 100%;
                    height: 100%;
                }
                canvas {
                    display: block;
                    width: 100%;
                    height: 100%;
                }
            </style>
        "#,
        );

        shadow.append_child(&canvas).unwrap();

        // Store the runner in the component
        let runner = eframe::WebRunner::new();
        self.runner = Some(runner.clone());

        // Start the eframe app asynchronously
        wasm_bindgen_futures::spawn_local(async move {
            let web_options = eframe::WebOptions::default();

            let start_result = runner
                .start(
                    canvas,
                    web_options,
                    Box::new(|cc| {
                        add_fonts_to_ctx(&cc.egui_ctx);
                        Ok(Box::new(AliasApp::default()))
                    }),
                )
                .await;

            if let Err(e) = start_result {
                web_sys::console::error_1(&format!("Failed to start eframe: {:?}", e).into());
            }
        });
    }

    fn disconnected(&mut self) {
        if let Some(runner) = self.runner.take() {
            runner.destroy();
        }
    }
}
