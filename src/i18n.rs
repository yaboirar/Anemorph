//! Local interface translations. Filename indexes and user-entered paths stay literal.
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, Id};
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};
pub const LANGUAGES: &[(&str, &str)] = &[
    ("en-US", "English (US)"),
    ("en-GB", "English (UK)"),
    ("ru", "Russian"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("tr", "Turkish"),
    ("de", "German"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh-Hans", "Chinese (Simplified)"),
    ("ar", "Arabic (Modern Standard)"),
];
const CODES: &[&str] = &["ru", "es", "pt", "tr", "de", "ja", "ko", "zh-Hans", "ar"];
fn arabic_display(text: &str) -> String {
    let shaped =
        arabic_reshaper::arabic_reshape(text).replace("{count}", "\u{2066}{count}\u{2069}");
    let bidi = unicode_bidi::BidiInfo::new(&shaped, Some(unicode_bidi::Level::rtl()));
    bidi.paragraphs
        .iter()
        .map(|p| bidi.reorder_line(p, p.range.clone()))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(['\u{2066}', '\u{2069}'], "")
}
static TEXT: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut table: HashMap<String, Vec<String>> =
        serde_json::from_str(include_str!("../assets/locales.json"))
            .expect("Embedded translations");
    for values in table.values_mut() {
        values[8] = arabic_display(&values[8]);
    }
    table
});
pub fn language_name(code: &str) -> &'static str {
    LANGUAGES
        .iter()
        .find(|(key, _)| *key == code)
        .unwrap_or(&LANGUAGES[0])
        .1
}
pub fn tr<'a>(ctx: &Context, key: &'a str) -> &'a str {
    let language = ctx
        .data(|d| d.get_temp::<usize>(Id::new("interface_language")))
        .unwrap_or(0);
    if language == 1 {
        return match key {
            "Highlight color" => "Highlight colour",
            "Hex color" => "Hex colour",
            "Unsaved color" => "Unsaved colour",
            "Enter a valid hex color." => "Enter a valid hex colour.",
            _ => key,
        };
    }
    if language < 2 {
        return key;
    }
    TEXT.get(key)
        .and_then(|v| v.get(language - 2))
        .map(String::as_str)
        .unwrap_or(key)
}
pub fn configure(ctx: &Context, code: &str) {
    let index = LANGUAGES.iter().position(|(c, _)| *c == code).unwrap_or(0);
    // Dictionary columns follow CODES, independently of chooser order.
    let language = CODES
        .iter()
        .position(|c| *c == code)
        .map(|i| i + 2)
        .unwrap_or(if code == "en-GB" { 1 } else { 0 });
    ctx.data_mut(|d| d.insert_temp(Id::new("interface_language"), language));
    let mut fonts = FontDefinitions::default();
    let root = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("Fonts");
    let mut brand = fonts.families[&FontFamily::Proportional].clone();
    if let Ok(bytes) = std::fs::read(root.join("arialbd.ttf")) {
        fonts
            .font_data
            .insert("arial-bold".into(), FontData::from_owned(bytes).into());
        brand.insert(0, "arial-bold".into());
    }
    fonts
        .families
        .insert(FontFamily::Name("brand".into()), brand);
    // Load one script font only when needed. Latin users do not retain CJK fonts.
    let candidates: &[&str] = match LANGUAGES[index].0 {
        "ja" => &["YuGothR.ttc", "meiryo.ttc", "msyh.ttc"],
        "ko" => &["malgun.ttf", "msyh.ttc"],
        "zh-Hans" => &["msyh.ttc", "simsun.ttc"],
        "ar" => &["arial.ttf", "segoeui.ttf"],
        _ => &[],
    };
    for filename in candidates {
        if let Ok(bytes) = std::fs::read(root.join(filename)) {
            fonts.font_data.insert(
                "interface-script".into(),
                FontData::from_owned(bytes).into(),
            );
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                fonts
                    .families
                    .get_mut(&family)
                    .unwrap()
                    .push("interface-script".into());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
    ctx.request_repaint();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_fonts_cover_translated_interface_glyphs() {
        for code in ["ru", "ja", "ko", "zh-Hans", "ar"] {
            let ctx = Context::default();
            configure(&ctx, code);
            let characters: Vec<char> = TEXT
                .keys()
                .flat_map(|key| tr(&ctx, key).chars())
                .filter(|c| !c.is_control())
                .collect();
            let _ = ctx.run(Default::default(), |ctx| {
                ctx.fonts_mut(|fonts| {
                    let font = eframe::egui::FontId::proportional(13.);
                    for ch in &characters {
                        assert!(fonts.has_glyph(&font, *ch), "Missing {ch:?} in {code}");
                    }
                });
            });
        }
    }
    #[test]
    fn every_language_has_every_phrase_and_keeps_placeholders() {
        for (key, values) in TEXT.iter() {
            assert_eq!(values.len(), CODES.len(), "{key}");
            for value in values {
                assert!(!value.trim().is_empty(), "{key}");
                if key.contains("{count}") {
                    assert!(value.contains("{count}"), "{value}");
                }
            }
        }
    }
    #[test]
    fn arabic_is_joined_and_english_uk_uses_colour() {
        let ctx = Context::default();
        configure(&ctx, "en-GB");
        assert_eq!(tr(&ctx, "Highlight color"), "Highlight colour");
        configure(&ctx, "ar");
        assert!(
            tr(&ctx, "Settings")
                .chars()
                .any(|c| ('\u{fe70}'..='\u{feff}').contains(&c))
        );
    }
    #[test]
    fn language_round_trips_and_unknown_language_falls_back() {
        for (code, _) in LANGUAGES {
            let ctx = Context::default();
            configure(&ctx, code);
            assert!(!tr(&ctx, "Home").is_empty());
        }
        assert_eq!(language_name("unknown"), "English (US)");
    }
}
