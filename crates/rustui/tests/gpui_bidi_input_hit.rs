use gpui::{LineLayout, WrappedGlyphCluster, px};

#[test]
fn mixed_direction_single_line_hits_follow_visual_clusters() {
    let layout = LineLayout {
        width: px(40.),
        len: 7,
        bidi_clusters: Some(vec![
            WrappedGlyphCluster {
                bytes: 4..6,
                x: px(0.),
                width: px(10.),
                rtl: true,
            },
            WrappedGlyphCluster {
                bytes: 2..4,
                x: px(10.),
                width: px(10.),
                rtl: true,
            },
            WrappedGlyphCluster {
                bytes: 0..2,
                x: px(20.),
                width: px(10.),
                rtl: true,
            },
            WrappedGlyphCluster {
                bytes: 6..7,
                x: px(30.),
                width: px(10.),
                rtl: false,
            },
        ]),
        ..Default::default()
    };

    assert_eq!(layout.closest_index_for_x(px(1.)), 6);
    assert_eq!(layout.closest_index_for_x(px(11.)), 4);
    assert_eq!(layout.closest_index_for_x(px(21.)), 2);
    assert_eq!(layout.closest_index_for_x(px(29.)), 0);
    assert_eq!(layout.closest_index_for_x(px(31.)), 6);
    assert_eq!(layout.x_for_index(0), px(30.));
    assert_eq!(layout.x_for_index(2), px(20.));
    assert_eq!(layout.index_for_x(px(25.)), Some(0));
}
