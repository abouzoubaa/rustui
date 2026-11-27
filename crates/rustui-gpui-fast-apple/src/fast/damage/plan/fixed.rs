// SPDX-License-Identifier: Apache-2.0
// RustUI: linear comparison for fixed-position updates before the pinned
// Longbridge planner (2002081b4a61249a1cef2b8a552f6ba0998d9f52,
// crates/gpui_windows/src/fast/partial.rs). Original RustUI optimization.
use super::{Area, MAX_DAMAGE, Plan, Policy, Sig, Trace, record_damage, tidy};

// Positions, clips and orders must match exactly. Any movement or reordering
// goes through the general multiset/scroll planner. Changed visual signatures
// and fresh atlas tiles redraw their old and new reach in paint order.
pub(super) fn plan(
    old: &[Sig],
    new: &[Sig],
    view: Area,
    trace: Option<&mut Trace>,
    policy: &mut Policy,
) -> Result<Option<Plan>, ()> {
    if old.len() != new.len() {
        return Err(());
    }
    let mut damage = Vec::new();
    for (old, new) in old.iter().zip(new) {
        if old.order != new.order || old.bounds != new.bounds || old.mask != new.mask {
            return Err(());
        }
        if old.key != new.key || new.fresh {
            damage.extend([old.reach, new.reach]);
        }
    }
    super::plan_stage(super::PlanStage::RectMerge);
    let trace = trace.map(|trace| {
        trace.route = "fixed";
        trace.changed(
            damage.iter().step_by(2).copied(),
            damage.iter().skip(1).step_by(2).copied(),
        );
        trace
    });
    if policy.rejects_area(damage.iter().copied(), view) {
        return Ok(None);
    }
    let damage = tidy(damage, &view);
    let total: f32 = damage.iter().map(Area::area).sum();
    record_damage(trace, &damage, total, view);
    super::plan_stage(super::PlanStage::BuildPlan);
    Ok((total <= MAX_DAMAGE * view.area()).then(|| Plan {
        shift: None,
        damage,
    }))
}

#[cfg(any(test, feature = "test-support"))]
pub(super) fn test_fixed() {
    let view = Area {
        x0: 0.,
        y0: 0.,
        x1: 1000.,
        y1: 800.,
    };
    let bounds = Area {
        x0: 10.,
        y0: 10.,
        x1: 20.,
        y1: 20.,
    };
    let old = Sig {
        key: 1,
        order: 1,
        bounds,
        mask: view,
        reach: bounds,
        plain: false,
        owner: 0,
        opaque: false,
        batch_rank: 0,
        fresh: false,
    };
    for new in [
        old,
        Sig { key: 2, ..old },
        Sig { fresh: true, ..old },
        Sig {
            key: 2,
            reach: Area {
                x0: 0.,
                x1: 30.,
                ..bounds
            },
            ..old
        },
    ] {
        assert_eq!(
            plan(&[old], &[new], view, None, &mut Policy::default()).unwrap(),
            super::general_plan(&[old], &[new], view, None, &mut Policy::default())
        );
    }
    assert!(plan(&[old], &[], view, None, &mut Policy::default()).is_err());
    assert!(
        plan(
            &[old],
            &[Sig { order: 2, ..old }],
            view,
            None,
            &mut Policy::default()
        )
        .is_err()
    );
    assert!(
        plan(
            &[old],
            &[Sig {
                bounds: Area { y0: 11., ..bounds },
                ..old
            }],
            view,
            None,
            &mut Policy::default(),
        )
        .is_err()
    );
    assert!(
        plan(
            &[old],
            &[Sig {
                mask: bounds,
                ..old
            }],
            view,
            None,
            &mut Policy::default(),
        )
        .is_err()
    );
    assert_eq!(
        plan(
            &[Sig { reach: view, ..old }],
            &[Sig {
                key: 2,
                reach: view,
                ..old
            }],
            view,
            None,
            &mut Policy::default(),
        )
        .unwrap(),
        None
    );
}
