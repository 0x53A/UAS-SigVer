pub mod app;
pub mod fonts;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod lib_mobile;

#[cfg(target_arch = "wasm32")]
pub mod font_wasm;
#[cfg(target_arch = "wasm32")]
pub mod wasm_component;
#[cfg(target_arch = "wasm32")]
pub mod lib_wasm;