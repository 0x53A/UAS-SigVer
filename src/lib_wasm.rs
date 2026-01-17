use wasm_bindgen::prelude::*;
use crate::wasm_component::UasSigverComponent;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    web_sys::console::log_1(&"UAS-SigVer web component module loaded".into());

    UasSigverComponent::setup();
}

