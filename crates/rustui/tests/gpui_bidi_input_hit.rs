use gpui::{BackendWrappedRow, LineLayout, WrappedGlyphCluster, WrappedLineLayout, point, px};
use std::sync::Arc;

#[test]
fn mixed_direction_single_line_hits_follow_visual_clusters() {
    let layout = WrappedLineLayout {
        unwrapped_layout: Arc::new(LineLayout {
            width: px(40.),
            len: 7,
            ..Default::default()
        }),
        wrap_width: Some(px(40.)),
        backend_rows: Some(vec![BackendWrappedRow {
            layout: LineLayout::default(),
            bytes: 0..7,
            clusters: vec![
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
            ],
        }]),
        ..Default::default()
    };

    assert_eq!(
        layout.closest_index_for_position(point(px(1.), px(1.)), px(10.)),
        Ok(6)
    );
    assert_eq!(
        layout.closest_index_for_position(point(px(11.), px(1.)), px(10.)),
        Ok(4)
    );
    assert_eq!(
        layout.closest_index_for_position(point(px(21.), px(1.)), px(10.)),
        Ok(2)
    );
    assert_eq!(
        layout.closest_index_for_position(point(px(29.), px(1.)), px(10.)),
        Ok(0)
    );
    assert_eq!(
        layout.closest_index_for_position(point(px(31.), px(1.)), px(10.)),
        Ok(6)
    );
    assert_eq!(layout.position_for_index(0, px(10.)).unwrap().x, px(30.));
    assert_eq!(layout.position_for_index(2, px(10.)).unwrap().x, px(20.));
    assert_eq!(
        layout.index_for_position(point(px(25.), px(1.)), px(10.)),
        Ok(0)
    );
}
