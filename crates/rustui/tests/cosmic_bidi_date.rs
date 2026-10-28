use cosmic_text::{
    Attrs, AttrsList, Ellipsize, Family, FontSystem, Hinting, ShapeBuffer, ShapeLine, Shaping, Wrap,
};

#[test]
fn unwrapped_bidi_dates_preserve_initial_digits_and_width() {
    let mut fonts = FontSystem::new_with_locale_and_db("en-US".into(), Default::default());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../assets/fonts/NotoSans-Regular.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../assets/fonts/NotoSansArabic.ttf").to_vec());
    let attrs = AttrsList::new(&Attrs::new().family(Family::Name("Noto Sans")));
    let mut width = Vec::new();
    for text in ["28\u{200f}/10\u{200f}/2026", "28/10/2026"] {
        let line = ShapeLine::new(&mut fonts, text, &attrs, Shaping::Advanced, 4);
        let mut rows = Vec::new();
        line.layout_to_buffer(
            &mut ShapeBuffer::default(),
            14.0,
            None,
            Wrap::None,
            Ellipsize::None,
            None,
            &mut rows,
            None,
            Hinting::Disabled,
        );
        assert_eq!(rows.len(), 1);
        let glyphs = &rows[0].glyphs;
        assert!(
            glyphs.iter().any(|glyph| glyph.start == 0),
            "missing first digit: {text:?}"
        );
        assert!(
            glyphs.iter().any(|glyph| glyph.start == 1),
            "missing second digit: {text:?}"
        );
        width.push(rows[0].w);
    }
    assert!(
        (width[0] - width[1]).abs() < 0.01,
        "direction marks must not change the measured width: {width:?}"
    );
    let text = "٢٨\u{200f}/١٠\u{200f}/٢٠٢٦";
    let attrs = AttrsList::new(&Attrs::new().family(Family::Name("Noto Sans Arabic")));
    let line = ShapeLine::new(&mut fonts, text, &attrs, Shaping::Advanced, 4);
    let mut rows = Vec::new();
    line.layout_to_buffer(
        &mut ShapeBuffer::default(),
        14.0,
        None,
        Wrap::None,
        Ellipsize::None,
        None,
        &mut rows,
        None,
        Hinting::Disabled,
    );
    assert_eq!(rows.len(), 1);
    assert!(rows[0].glyphs.iter().any(|glyph| glyph.start == 0));
    assert!(
        rows[0]
            .glyphs
            .iter()
            .any(|glyph| glyph.start == '٢'.len_utf8())
    );
}
