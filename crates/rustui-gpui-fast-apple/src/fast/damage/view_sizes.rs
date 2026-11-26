// SPDX-License-Identifier: Apache-2.0
//! Separate sizing capture; bounded by actual view slots. Disabled for timing.
use gpui::Scene;
use std::{cell::RefCell, sync::OnceLock};

fn count(scene: &Scene, slots: &mut Vec<u32>) {
    let counts = [
        scene.quads.len(),
        scene.shadows.len(),
        scene.underlines.len(),
        scene.monochrome_sprites.len(),
        scene.subpixel_sprites.len(),
        scene.polychrome_sprites.len(),
        scene.paths.len(),
    ];
    for (kind, len) in counts.into_iter().enumerate() {
        for index in 0..len {
            let view = scene.fast_damage_view(kind, index);
            let slot = view as u32 as usize;
            if slots.len() <= slot {
                slots.resize(slot + 1, 0);
            }
            slots[slot] += 1;
        }
    }
    for glyph in &scene.polychrome_sprites {
        if let Some(child) = crate::fast::rounded_clip::group(scene, glyph.tile.texture_id)
            .and_then(|frame| frame.content.scene())
        {
            count(child, slots);
        }
    }
}

pub(super) fn sample(scene: &Scene) {
    static ACTIVE: OnceLock<bool> = OnceLock::new();
    if !*ACTIVE
        .get_or_init(|| std::env::var_os("RUSTUI_G_R2_VIEW_SIZES").as_deref() == Some("1".as_ref()))
    {
        return;
    }
    thread_local! { static SLOTS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) }; }
    SLOTS.with(|slots| {
        let mut slots = slots.borrow_mut();
        slots.clear();
        count(scene, &mut slots);
        for (slot, count) in slots.iter().enumerate().filter(|(_, n)| **n != 0) {
            eprintln!("RUSTUI_G_R2_VIEW_SIZE {slot} {count}");
        }
    });
}
