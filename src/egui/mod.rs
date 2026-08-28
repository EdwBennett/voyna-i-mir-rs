#[cfg(target_arch = "wasm32")]
pub mod hello;
pub mod page2;
pub mod select_word;

use eframe::egui;

/// Registers Noto Sans as the primary font for both the proportional and
/// monospace font families, ahead of egui's bundled defaults (Ubuntu-Light /
/// Hack). Those defaults don't cover the IPA primary-stress mark (U+02C8
/// 'ˈ', used ahead of a syllable to mark word stress in the `words` excerpt
/// text) and, in Hack's case, no Cyrillic either - so without this, stress
/// marks in the Russian text render as missing glyphs.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSans-Regular.ttf"
        ))),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans".to_owned());
    ctx.set_fonts(fonts);
}
