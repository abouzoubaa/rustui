//! Pure edit planning for the native IME owner's verified UTF-16 text window.
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ImeEdit {
    pub(crate) range: Range<usize>,
    pub(crate) text: String,
}

/// A DOM edit is relative to the stored text window, never a newer app selection.
/// The caller must verify that the application still contains `old` at `start`.
pub(crate) fn plan_ime_edit(
    old: &str,
    new: &str,
    caret: Option<usize>,
    start: usize,
) -> Option<ImeEdit> {
    if old == new {
        return None;
    }
    let old: Vec<u16> = old.encode_utf16().collect();
    let new: Vec<u16> = new.encode_utf16().collect();
    let anchored_suffix = caret
        .filter(|caret| *caret <= new.len())
        .map(|caret| new.len() - caret)
        .filter(|suffix| {
            *suffix <= old.len() && old[old.len() - suffix..] == new[new.len() - suffix..]
        });
    let mut suffix = anchored_suffix.unwrap_or_else(|| {
        old.iter()
            .rev()
            .zip(new.iter().rev())
            .take_while(|(a, b)| a == b)
            .count()
    });
    // Never split a surrogate pair at either end of the replacement.
    while suffix > 0
        && (splits_pair(&old, old.len() - suffix) || splits_pair(&new, new.len() - suffix))
    {
        suffix -= 1;
    }
    let mut prefix = old
        .iter()
        .zip(&new)
        .take_while(|(a, b)| a == b)
        .count()
        .min(old.len() - suffix)
        .min(new.len() - suffix);
    while prefix > 0 && (splits_pair(&old, prefix) || splits_pair(&new, prefix)) {
        prefix -= 1;
    }
    Some(ImeEdit {
        range: start.checked_add(prefix)?..start.checked_add(old.len() - suffix)?,
        text: String::from_utf16(&new[prefix..new.len() - suffix]).ok()?,
    })
}
fn splits_pair(units: &[u16], index: usize) -> bool {
    index > 0
        && index < units.len()
        && (0xD800..=0xDBFF).contains(&units[index - 1])
        && (0xDC00..=0xDFFF).contains(&units[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_selection_deletion_ignores_new_application_caret() {
        // App caret may already have moved to zero; the stored window is fixed.
        let edit = plan_ime_edit("te", "", Some(0), 0).unwrap();
        assert_eq!(
            edit,
            ImeEdit {
                range: 0..2,
                text: String::new()
            }
        );
    }
    #[test]
    fn empty_replacement_deletes_entire_window() {
        assert_eq!(
            plan_ime_edit("tre", "", Some(0), 40),
            Some(ImeEdit {
                range: 40..43,
                text: String::new()
            })
        );
    }
    #[test]
    fn ordinary_partial_deletion_preserves_absolute_alignment() {
        assert_eq!(
            plan_ime_edit("tree", "tre", Some(3), 100),
            Some(ImeEdit {
                range: 103..104,
                text: String::new()
            })
        );
    }
    #[test]
    fn repeated_text_uses_post_edit_caret_to_anchor_insertion() {
        assert_eq!(
            plan_ime_edit("pact", "pactor pact", Some(7), 0),
            Some(ImeEdit {
                range: 0..0,
                text: "pactor ".into()
            })
        );
    }
    #[test]
    fn supplementary_scalar_deletion_uses_utf16_offsets() {
        assert_eq!(
            plan_ime_edit("a😀b", "ab", Some(1), 100),
            Some(ImeEdit {
                range: 101..103,
                text: String::new()
            })
        );
    }
    #[test]
    fn supplementary_scalar_replacement_does_not_split_shared_surrogates() {
        assert_eq!(
            plan_ime_edit("😀", "😁", Some(2), 0),
            Some(ImeEdit {
                range: 0..2,
                text: "😁".into()
            })
        );
    }
    #[test]
    fn identical_text_needs_no_edit() {
        assert_eq!(plan_ime_edit("", "", Some(0), 0), None);
    }
    #[test]
    fn offset_overflow_is_rejected() {
        assert_eq!(plan_ime_edit("a", "", Some(0), usize::MAX), None);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SelectionPlan {
    /// An unchanged DOM selection must not overwrite a newer GPUI selection.
    KeepApplication,
    Import(Range<usize>),
    Reject,
}

/// Resolve native selection movement against the same absolute text baseline.
/// Callers verify baseline text first; independently changed selections conflict.
pub(crate) fn plan_ime_selection(
    text: &str,
    start: usize,
    stored: (u32, u32),
    live: (u32, u32),
    application: Range<usize>,
) -> SelectionPlan {
    let units: Vec<u16> = text.encode_utf16().collect();
    let map = |(left, right): (u32, u32)| {
        let (left, right) = (left as usize, right as usize);
        if left > right
            || right > units.len()
            || splits_pair(&units, left)
            || splits_pair(&units, right)
        {
            return None;
        }
        Some(start.checked_add(left)?..start.checked_add(right)?)
    };
    let (Some(baseline), Some(native)) = (map(stored), map(live)) else {
        return SelectionPlan::Reject;
    };
    if live == stored && application != baseline {
        return SelectionPlan::KeepApplication;
    }
    if application == baseline || application == native {
        SelectionPlan::Import(native)
    } else {
        SelectionPlan::Reject
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;
    #[test]
    fn native_select_all_before_next_key_imports_full_absolute_range() {
        assert_eq!(
            plan_ime_selection("tree", 40, (4, 4), (0, 4), 44..44),
            SelectionPlan::Import(40..44)
        );
    }
    #[test]
    fn gpui_select_all_before_mirror_sync_is_preserved() {
        assert_eq!(
            plan_ime_selection("tree", 40, (4, 4), (4, 4), 40..44),
            SelectionPlan::KeepApplication
        );
    }
    #[test]
    fn already_imported_native_selection_is_idempotent() {
        assert_eq!(
            plan_ime_selection("tree", 40, (4, 4), (0, 4), 40..44),
            SelectionPlan::Import(40..44)
        );
    }
    #[test]
    fn conflicting_model_and_native_selection_is_rejected() {
        assert_eq!(
            plan_ime_selection("tree", 40, (4, 4), (0, 4), 41..42),
            SelectionPlan::Reject
        );
    }
    #[test]
    fn native_partial_selection_is_independent_of_model_caret_delta() {
        assert_eq!(
            plan_ime_selection("tree", 100, (4, 4), (1, 3), 104..104),
            SelectionPlan::Import(101..103)
        );
    }
    #[test]
    fn supplementary_scalar_selection_maps_utf16_and_rejects_split_pairs() {
        assert_eq!(
            plan_ime_selection("a😀b", 10, (4, 4), (1, 3), 14..14),
            SelectionPlan::Import(11..13)
        );
        assert_eq!(
            plan_ime_selection("a😀b", 10, (4, 4), (2, 3), 14..14),
            SelectionPlan::Reject
        );
    }
    #[test]
    fn invalid_native_ranges_and_offset_overflow_are_rejected() {
        for live in [(3, 1), (0, 5)] {
            assert_eq!(
                plan_ime_selection("tree", 0, (4, 4), live, 4..4),
                SelectionPlan::Reject
            );
        }
        assert_eq!(
            plan_ime_selection("tree", usize::MAX, (4, 4), (0, 4), 0..0),
            SelectionPlan::Reject
        );
    }
    #[test]
    fn empty_native_and_application_selection_is_stable() {
        assert_eq!(
            plan_ime_selection("", 0, (0, 0), (0, 0), 0..0),
            SelectionPlan::Import(0..0)
        );
    }
}

#[cfg(test)]
mod replacement_tests {
    use super::*;
    #[test]
    fn settled_native_replacement_removes_old_window() {
        assert_eq!(
            plan_ime_edit("tree", "button", Some(6), 0),
            Some(ImeEdit {
                range: 0..4,
                text: "button".into()
            })
        );
    }
    #[test]
    fn repeated_native_replacement_does_not_append_to_existing_window() {
        assert_eq!(
            plan_ime_edit("treebutton", "button", Some(6), 0),
            Some(ImeEdit {
                range: 0..10,
                text: "button".into()
            })
        );
    }
}
