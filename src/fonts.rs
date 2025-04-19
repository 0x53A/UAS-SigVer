// see https://github.com/emilk/egui/blob/master/crates/epaint_default_fonts/src/lib.rs

/// A typeface designed for source code.
///
/// Hack is designed to be a workhorse typeface for source code. It has deep
/// roots in the free, open source typeface community and expands upon the
/// contributions of the [Bitstream Vera](https://www.gnome.org/fonts/) and
/// [DejaVu](https://dejavu-fonts.github.io/) projects.  The large x-height +
/// wide aperture + low contrast design make it legible at commonly used source
/// code text sizes with a sweet spot that runs in the 8 - 14 range.
///
/// See [the Hack repository](https://github.com/source-foundry/Hack) for more
/// information.
// pub const HACK_REGULAR: &[u8] = include_bytes!("../fonts/Hack-Regular.ttf");

/// A typeface designed for use by Ubuntu.
///
/// The Ubuntu typeface has been specially created to complement the Ubuntu tone
/// of voice. It has a contemporary style and contains characteristics unique to
/// the Ubuntu brand that convey a precise, reliable and free attitude.
///
/// See [Ubuntu design](https://design.ubuntu.com/font) for more information.
#[cfg(not(target_arch = "wasm32"))]
pub const UBUNTU_LIGHT: &[u8] = include_bytes!("../fonts/Ubuntu-Light.ttf");

#[cfg(target_arch = "wasm32")]
pub const UBUNTU_LIGHT_GZIP: &[u8] = include_bytes!("../fonts/Ubuntu-Light.ttf.gz");
