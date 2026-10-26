#[cfg(test)]
mod tests {
    use cosmic_text::{
        Attrs, AttrsList, Ellipsize, FontSystem, Hinting, ShapeBuffer, ShapeLine, Shaping, Wrap,
    };

    #[test]
    fn mixed_arabic_wrap_rows_retain_all_text_and_local_geometry() {
        let text = "مثال RustUI لـ Button يتضمن عدادًا مباشرًا وإجرائي Increment وReset وحالة Unavailable معطلة؛ زال القبول المرئي واختبارات لوحة المفاتيح وإمكانية الوصول بحاجة إلى التحقق الأصلي.";
        let mut fonts = FontSystem::new();
        let attrs = AttrsList::new(&Attrs::new());
        let line = ShapeLine::new(&mut fonts, text, &attrs, Shaping::Advanced, 4);
        let mut rows = Vec::new();
        line.layout_to_buffer(
            &mut ShapeBuffer::default(),
            14.0,
            Some(180.0),
            Wrap::WordOrGlyph,
            Ellipsize::None,
            None,
            &mut rows,
            None,
            Hinting::Disabled,
        );
        assert!(rows.len() > 1, "expected wrapped rows: {rows:?}");
        assert!(
            rows.iter()
                .all(|row| !row.glyphs.is_empty() && row.w <= 181.0)
        );
        let starts: Vec<usize> = rows
            .iter()
            .flat_map(|row| row.glyphs.iter().map(|glyph| glyph.start))
            .collect();
        assert!(starts.iter().any(|&start| start < text.len() / 4));
        assert!(starts.iter().any(|&start| start > text.len() * 3 / 4));
        let ranges: Vec<(usize, usize)> = rows
            .iter()
            .map(|row| {
                (
                    row.glyphs.iter().map(|g| g.start).min().unwrap(),
                    row.glyphs.iter().map(|g| g.end).max().unwrap(),
                )
            })
            .collect();
        assert!(
            ranges.windows(2).all(|pair| pair[0].1 <= pair[1].0),
            "row logical ranges overlap or reverse: {ranges:?}"
        );
        let glyphs: Vec<_> = rows.iter().flat_map(|row| &row.glyphs).collect();
        assert!(glyphs.iter().any(|glyph| glyph.level.is_rtl()));
        assert!(glyphs.iter().any(|glyph| glyph.level.is_ltr()));
        for glyph in &glyphs {
            assert!(glyph.start < glyph.end);
            assert!(text.is_char_boundary(glyph.start));
            assert!(text.is_char_boundary(glyph.end));
        }
        for (index, ch) in text.char_indices() {
            if !ch.is_whitespace() {
                assert!(
                    glyphs
                        .iter()
                        .any(|glyph| glyph.start <= index && index < glyph.end),
                    "visible character at byte {index} was dropped"
                );
            }
        }
        for row in &rows {
            assert!(
                row.glyphs.iter().all(|g| g.x >= -1.0 && g.x + g.w <= 181.0),
                "row x outside requested width: {row:?}"
            );
        }
    }
}
