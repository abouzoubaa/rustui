// SPDX-License-Identifier: Apache-2.0
//! Opt-in measurements of the planner's decisions. The planner owns all decisions.
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::SystemTime,
};

use super::plan::Area;

pub(super) struct Trace {
    sequence: usize,
    pub reason: &'static str,
    pub route: &'static str,
    pub coarse_rejections: usize,
    pub order_conflict: Option<[[u32; 2]; 2]>,
    old: Vec<[f32; 4]>,
    new: Vec<[f32; 4]>,
    damage: Vec<[f32; 4]>,
    damage_area: f32,
    old_count: usize,
    new_count: usize,
}

impl Trace {
    pub fn start() -> Option<Self> {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
        ENABLED
            .get_or_init(|| std::env::var_os("RUSTUI_G_R1_TRACE").as_deref() == Some("1".as_ref()))
            .then(|| Self::new(SEQUENCE.fetch_add(1, Ordering::Relaxed)))
    }

    fn new(sequence: usize) -> Self {
        Self {
            sequence,
            reason: "planned",
            route: "preflight",
            coarse_rejections: 0,
            order_conflict: None,
            old: Vec::new(),
            new: Vec::new(),
            damage: Vec::new(),
            damage_area: 0.,
            old_count: 0,
            new_count: 0,
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn test_sample() -> Self {
        Self::new(0)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn damage_was_built(&self) -> bool {
        !self.damage.is_empty()
    }

    pub fn changed(&mut self, old: impl Iterator<Item = Area>, new: impl Iterator<Item = Area>) {
        let sampled = self.sequence.is_multiple_of(32);
        for area in old {
            self.old_count += 1;
            if sampled && self.old.len() < 64 {
                self.old.push(rect(area));
            }
        }
        for area in new {
            self.new_count += 1;
            if sampled && self.new.len() < 64 {
                self.new.push(rect(area));
            }
        }
    }

    pub fn damage(&mut self, areas: &[Area]) {
        self.damage_area = areas.iter().map(Area::area).sum();
        if self.sequence.is_multiple_of(32) {
            self.damage.extend(areas.iter().copied().map(rect));
        }
    }

    pub fn emit(self, mode: &str, view: Area, bounds: Option<Area>) {
        let sequence = self.sequence;
        let epoch_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.;
        let damage_area = self.damage_area;
        let view_area = view.area();
        let bounds = bounds.map_or([0.; 4], rect);
        let order_conflict = self.order_conflict.unwrap_or([[0; 2]; 2]);
        super::log::line(format_args!(
            r#"RUSTUI_FRAME_LOG {{"event":"damage_diagnosis","cpu_ms":0.0,"epoch_ms":{epoch_ms:.6},"sequence":{sequence},"mode":"{mode}","reason":"{}","route":"{}","sample":{},"coarse_rejections":{},"view_area":{view_area},"damage_area":{damage_area},"bounds":{bounds:?},"conflicting_orders":{order_conflict:?},"old_count":{},"new_count":{},"old":{:?},"new":{:?},"damage":{:?}}}"#,
            self.reason,
            self.route,
            sequence.is_multiple_of(32),
            self.coarse_rejections,
            self.old_count,
            self.new_count,
            self.old,
            self.new,
            self.damage,
        ));
    }
}

fn rect(area: Area) -> [f32; 4] {
    [area.x0, area.y0, area.x1, area.y1]
}
