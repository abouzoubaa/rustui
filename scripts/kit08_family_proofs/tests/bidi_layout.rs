use gpui::{LineLayout, WrappedGlyphCluster, px};

#[test]
fn single_line_pointer_hit_uses_visual_bidi_clusters() {
    let layout = LineLayout {
        font_size: px(10.),
        width: px(40.),
        ascent: px(8.),
        descent: px(2.),
        runs: Vec::new(),
        len: 7,
        bidi_clusters: Some(vec![
            WrappedGlyphCluster { bytes: 4..6, x: px(0.), width: px(10.), rtl: true },
            WrappedGlyphCluster { bytes: 2..4, x: px(10.), width: px(10.), rtl: true },
            WrappedGlyphCluster { bytes: 0..2, x: px(20.), width: px(10.), rtl: true },
            WrappedGlyphCluster { bytes: 6..7, x: px(30.), width: px(10.), rtl: false },
        ]),
    };
    assert_eq!(layout.closest_index_for_x(px(1.)), 6);
}
