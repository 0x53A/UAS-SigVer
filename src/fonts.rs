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
#[cfg(feature = "font_ubuntu_light")]
pub const UBUNTU_LIGHT: &[u8] = include_bytes!("../fonts/Ubuntu-Light.ttf");

#[cfg(feature = "font_ubuntu_light_compressed")]
pub static UBUNTU_LIGHT: Option<Box<[u8]>> = None;
#[cfg(feature = "font_ubuntu_light_compressed")]
pub const UBUNTU_LIGHT_GZIP: &[u8] = include_bytes!("../fonts/Ubuntu-Light.ttf.gz");

#[cfg(feature = "font_hack")]
pub const HACK: &[u8] = include_bytes!("../fonts/Hack-Regular.ttf");

#[cfg(feature = "font_berkeley_mono")]
pub const BERKELEY_MONO: &[u8] = include_bytes!(
    "../fonts/berkeley-mono/v2/250521L627KKV86L/TX-02-Y6N88QJ9/BerkeleyMono-Regular.ttf"
);



#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn add_fonts_to_ctx(egui_ctx: &egui::Context) {
    use std::{collections::BTreeMap, sync::Arc};
    use egui::FontData;
    use egui::FontFamily;
    use egui::FontDefinitions;

    let mut font_data: BTreeMap<String, Arc<FontData>> = BTreeMap::new();

    let mut families = BTreeMap::new();

    #[cfg(feature = "font_hack")]
    font_data.insert(
        "Hack".to_owned(),
        Arc::new(FontData::from_static(crate::fonts::HACK)),
    );

    // // Some good looking emojis. Use as first priority:
    // font_data.insert(
    //     "NotoEmoji-Regular".to_owned(),
    //     Arc::new(FontData::from_static(crate::fonts::NOTO_EMOJI_REGULAR).tweak(FontTweak {
    //         scale: 0.81, // Make smaller
    //         ..Default::default()
    //     })),
    // );

    #[cfg(feature = "font_ubuntu_light")]
    font_data.insert(
        "Ubuntu-Light".to_owned(),
        Arc::new(FontData::from_static(crate::fonts::UBUNTU_LIGHT)),
    );

    #[cfg(feature = "font_ubuntu_light_compressed")]
    font_data.insert(
        "Ubuntu-Light".to_owned(),
        Arc::new(FontData::from_owned(crate::fonts::UBUNTU_LIGHT.to_vec())),
    );

    // // Bigger emojis, and more. <http://jslegers.github.io/emoji-icon-font/>:
    // font_data.insert(
    //     "emoji-icon-font".to_owned(),
    //     Arc::new(FontData::from_static(crate::fonts::EMOJI_ICON).tweak(FontTweak {
    //         scale: 0.90, // Make smaller
    //         ..Default::default()
    //     })),
    // );

    #[cfg(feature = "font_berkeley_mono")]
    font_data.insert(
        "BerkeleyMono".to_owned(),
        Arc::new(FontData::from_static(crate::fonts::BERKELEY_MONO)),
    );

    families.insert(
        FontFamily::Monospace,
        vec![
            #[cfg(feature = "font_berkeley_mono")]
            "BerkeleyMono".to_owned(),
            #[cfg(feature = "font_hack")]
            "Hack".to_owned(),
            #[cfg(feature = "font_ubuntu_light")]
            "Ubuntu-Light".to_owned(),
            // "NotoEmoji-Regular".to_owned(),
            // "emoji-icon-font".to_owned(),
        ],
    );
    families.insert(
        FontFamily::Proportional,
        vec![
            #[cfg(feature = "font_berkeley_mono")]
            "BerkeleyMono".to_owned(),
            #[cfg(feature = "font_ubuntu_light")]
            "Ubuntu-Light".to_owned(),
            #[cfg(feature = "font_hack")]
            "Hack".to_owned(),
            // "NotoEmoji-Regular".to_owned(),
            // "emoji-icon-font".to_owned(),
        ],
    );

    let fd = FontDefinitions {
        font_data,
        families,
    };

    egui_ctx.set_fonts(fd);
}
