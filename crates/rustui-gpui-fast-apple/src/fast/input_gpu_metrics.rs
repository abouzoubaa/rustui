// G5-L6 diagnostic. Bounds represent raster work proxies, not shader-fragment counters.
use gpui::{Bounds, ScaledPixels, Scene, TransformationMatrix, point};
use std::{cell::Cell, sync::OnceLock};

thread_local! {
    static CALLS: Cell<Option<u64>> = const { Cell::new(None) };
}

pub(crate) struct Scope(bool);
impl Scope {
    pub(crate) fn enter() -> Self {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        let enabled = gpui::fast_latency_enabled()
            && *ENABLED.get_or_init(|| {
                std::env::var_os("RUSTUI_G5_GPU_DENSITY").as_deref() == Some("1".as_ref())
            });
        if enabled {
            CALLS.set(Some(0));
        }
        Self(enabled)
    }

    pub(crate) fn emit(&self, scene: &Scene, width: u32, height: u32) {
        if !self.0 {
            return;
        }
        let Some(frame) = gpui::fast_latency_frame() else {
            return;
        };
        let mut coverage = Coverage::new(width, height);
        let mut areas = [0.; 8];
        macro_rules! count {
            ($index:expr, $items:expr, $transform:expr) => {
                for item in $items {
                    areas[$index] +=
                        coverage.add(item.bounds, item.content_mask.bounds, $transform(item));
                }
            };
        }
        count!(0, &scene.quads, |_| None);
        count!(1, &scene.shadows, |_| None);
        count!(2, &scene.paths, |_| None);
        count!(3, &scene.underlines, |_| None);
        count!(
            4,
            &scene.monochrome_sprites,
            |item: &gpui::MonochromeSprite| Some(item.transformation)
        );
        count!(5, &scene.subpixel_sprites, |item: &gpui::SubpixelSprite| {
            Some(item.transformation)
        });
        count!(6, &scene.polychrome_sprites, |_| None);
        count!(7, &scene.surfaces, |_| None);

        let report = |kind, values| gpui::fast_latency_config(frame.window(), kind, values);
        report(
            "g56_gpu_counts_shapes",
            [
                scene.quads.len() as f64,
                scene.shadows.len() as f64,
                scene.paths.len() as f64,
                scene.underlines.len() as f64,
            ],
        );
        report(
            "g56_gpu_counts_sprites",
            [
                scene.monochrome_sprites.len() as f64,
                scene.subpixel_sprites.len() as f64,
                scene.polychrome_sprites.len() as f64,
                scene.surfaces.len() as f64,
            ],
        );
        report(
            "g56_gpu_area_shapes",
            [areas[0], areas[1], areas[2], areas[3]],
        );
        report(
            "g56_gpu_area_sprites",
            [areas[4], areas[5], areas[6], areas[7]],
        );
        report(
            "g56_gpu_work",
            [
                CALLS.get().unwrap_or(0) as f64,
                f64::from(width) * f64::from(height),
                areas.iter().sum(),
                coverage.occupied_area(),
            ],
        );
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.0 {
            CALLS.set(None);
        }
    }
}

#[inline]
pub(crate) fn draw_call() {
    if let Some(count) = CALLS.get() {
        CALLS.set(Some(count + 1));
    }
}

const TILE: usize = 16;
struct Coverage {
    width: usize,
    height: usize,
    columns: usize,
    occupied: Vec<bool>,
}

impl Coverage {
    fn new(width: u32, height: u32) -> Self {
        let width = width as usize;
        let height = height as usize;
        let columns = width.div_ceil(TILE);
        Self {
            width,
            height,
            columns,
            occupied: vec![false; columns * height.div_ceil(TILE)],
        }
    }

    fn add(
        &mut self,
        bounds: Bounds<ScaledPixels>,
        clip: Bounds<ScaledPixels>,
        transform: Option<TransformationMatrix>,
    ) -> f64 {
        let (x0, y0, x1, y1) = (
            bounds.origin.x.0,
            bounds.origin.y.0,
            bounds.origin.x.0 + bounds.size.width.0,
            bounds.origin.y.0 + bounds.size.height.0,
        );
        let (mut left, mut top, mut right, mut bottom) = (x0, y0, x1, y1);
        if let Some(matrix) = transform.filter(|m| *m != TransformationMatrix::unit()) {
            left = f32::INFINITY;
            top = f32::INFINITY;
            right = f32::NEG_INFINITY;
            bottom = f32::NEG_INFINITY;
            for (x, y) in [(x0, y0), (x0, y1), (x1, y0), (x1, y1)] {
                let p = matrix.apply(point(gpui::px(x), gpui::px(y)));
                let (x, y) = (f32::from(p.x), f32::from(p.y));
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
            }
        }
        left = left.max(clip.origin.x.0).max(0.);
        top = top.max(clip.origin.y.0).max(0.);
        right = right
            .min(clip.origin.x.0 + clip.size.width.0)
            .min(self.width as f32);
        bottom = bottom
            .min(clip.origin.y.0 + clip.size.height.0)
            .min(self.height as f32);
        if !(left < right && top < bottom) {
            return 0.;
        }
        let x = (left as usize / TILE)..(right.ceil() as usize).div_ceil(TILE);
        for y in (top as usize / TILE)..(bottom.ceil() as usize).div_ceil(TILE) {
            self.occupied[y * self.columns + x.start..y * self.columns + x.end].fill(true);
        }
        f64::from(right - left) * f64::from(bottom - top)
    }

    fn occupied_area(&self) -> f64 {
        self.occupied
            .iter()
            .enumerate()
            .filter(|(_, occupied)| **occupied)
            .map(|(index, _)| {
                let x = (index % self.columns) * TILE;
                let y = (index / self.columns) * TILE;
                (self.width - x).min(TILE) * (self.height - y).min(TILE)
            })
            .sum::<usize>() as f64
    }
}
