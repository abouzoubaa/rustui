// SPDX-License-Identifier: Apache-2.0
// Adapted from longbridge/gpui-fast, commit 2002081b4a61249a1cef2b8a552f6ba0998d9f52,
// crates/gpui_windows/src/fast/partial.rs (Uploads and take_uploads).
// Copyright 2022 - 2025 Zed Industries, Inc.; Copyright 2026 Longbridge.
// RustUI modifications: bounded, non-draining observations for shared Metal atlases.
use super::plan::Uploads;
use gpui::{AtlasTextureId, AtlasTile};
use std::collections::{HashSet, VecDeque};

pub(crate) struct Snapshot {
    pub(crate) generation: u64,
    /// None means that removals, clearing, or an expired journal require a whole draw.
    pub(crate) tiles: Option<HashSet<(AtlasTextureId, u32)>>,
}

#[derive(Default)]
pub(crate) struct Tracker {
    generation: u64,
    lost_through: u64,
    events: VecDeque<(u64, AtlasTile)>,
}
impl Tracker {
    const MOST: usize = 4096;
    pub(crate) fn record(&mut self, tile: &AtlasTile) {
        self.generation += 1;
        self.events.push_back((self.generation, *tile));
        if self.events.len() > Self::MOST {
            self.lost_through = self.events.pop_front().unwrap().0;
        }
    }
    pub(crate) fn clear(&mut self) {
        self.generation += 1;
        self.lost_through = self.generation;
        self.events.clear();
    }
    pub(crate) fn since(&self, after: u64) -> Snapshot {
        if after == self.generation {
            return Snapshot {
                generation: self.generation,
                tiles: Some(HashSet::new()),
            };
        }
        let mut uploads = Uploads::default();
        if after < self.lost_through || after > self.generation {
            uploads.clear();
        } else {
            for (generation, tile) in &self.events {
                if *generation > after {
                    uploads.record(tile);
                }
            }
        }
        Snapshot {
            generation: self.generation,
            tiles: uploads.take(),
        }
    }
}

pub(crate) fn dynamic(id: AtlasTextureId) -> bool {
    gpui::DynamicTextureId::from_atlas_texture_id(id).is_some()
}

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn test_upload_observers() {
    let tile = gpui::AtlasTile {
        texture_id: AtlasTextureId {
            index: 3,
            kind: gpui::AtlasTextureKind::Monochrome,
        },
        tile_id: gpui::TileId(9),
        bounds: gpui::Bounds {
            origin: gpui::point(gpui::DevicePixels(0), gpui::DevicePixels(0)),
            size: gpui::size(gpui::DevicePixels(4), gpui::DevicePixels(4)),
        },
        padding: 0,
    };
    let mut tracker = Tracker::default();
    tracker.record(&tile);
    let first = tracker.since(0);
    let second = tracker.since(0);
    assert_eq!(
        first.tiles, second.tiles,
        "one renderer must not consume another renderer's uploads"
    );
    assert!(
        first
            .tiles
            .unwrap()
            .contains(&(tile.texture_id, tile.tile_id.0))
    );
    assert!(tracker.since(first.generation).tiles.unwrap().is_empty());
    tracker.clear();
    assert!(tracker.since(first.generation).tiles.is_none());
    let floor = tracker.generation;
    tracker.record(&tile);
    assert_eq!(tracker.since(floor).tiles.unwrap().len(), 1);
    for _ in 0..=Tracker::MOST {
        tracker.record(&tile);
    }
    assert!(tracker.since(floor).tiles.is_none());
    assert!(tracker.since(tracker.generation).tiles.unwrap().is_empty());
    assert!(tracker.events.len() <= Tracker::MOST);
}

#[cfg(test)]
mod tests {
    #[test]
    fn upload_journal_preserves_multiple_observers_and_reports_loss() {
        super::test_upload_observers();
    }
}

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn test_atlas_hooks() -> anyhow::Result<()> {
    use gpui::PlatformAtlas as _;
    objc2::rc::autoreleasepool(|_| {
        let renderer = crate::metal_renderer::MetalRenderer::new_headless(Default::default())?;
        let atlas = renderer.sprite_atlas();
        let size = gpui::size(gpui::DevicePixels(4), gpui::DevicePixels(4));
        let key = gpui::AtlasKey::Svg(gpui::RenderSvgParams {
            path: "stage-g-upload-proof".into(),
            size,
        });
        let tile = atlas
            .get_or_insert_with(key.clone(), &mut || {
                Ok(Some((size, std::borrow::Cow::Owned(vec![128; 16]))))
            })?
            .expect("uploaded tile");
        let first = atlas.damage_uploads(0);
        let second = atlas.damage_uploads(0);
        assert_eq!(first.tiles, second.tiles);
        assert!(
            first
                .tiles
                .as_ref()
                .unwrap()
                .contains(&(tile.texture_id, tile.tile_id.0))
        );
        assert!(!dynamic(tile.texture_id));
        atlas.remove(&key);
        assert!(atlas.damage_uploads(first.generation).tiles.is_none());
        Ok(())
    })
}
