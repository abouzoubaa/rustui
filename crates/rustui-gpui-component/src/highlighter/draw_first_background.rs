// RustUI G5-3b: snapshot parsing runs on GPUI's background executor.
use super::super::highlighter::{InjectionLayer, InjectionParseData};
use super::*;

pub(super) enum FoldCandidates {
    Owned(Vec<FoldRange>),
    Cached(Arc<Vec<FoldRange>>),
}

impl FoldCandidates {
    pub(super) fn into_owned(self) -> Vec<FoldRange> {
        match self {
            Self::Owned(ranges) => ranges,
            Self::Cached(ranges) => Arc::unwrap_or_clone(ranges),
        }
    }
}
pub(super) fn parse_snapshot(
    language: SharedString,
    old_tree: Option<tree_sitter::Tree>,
    injection_data: Option<InjectionParseData>,
    text: Rope,
    folding: bool,
    parse_cancel: Arc<AtomicBool>,
    highlight_query: Option<Arc<tree_sitter::Query>>,
) -> Option<(
    tree_sitter::Tree,
    Vec<InjectionLayer>,
    FoldCandidates,
    Option<Arc<super::super::highlighter::draw_first_cache::Cache>>,
)> {
    parse_scoped(
        language,
        old_tree,
        injection_data,
        text,
        folding,
        parse_cancel,
        highlight_query,
        None,
        None,
        Vec::new(),
        super::super::highlighter::rust_empty::State::inert(),
        0,
    )
}
#[allow(clippy::too_many_arguments)]
pub(super) fn parse_scoped(
    language: SharedString,
    old_tree: Option<tree_sitter::Tree>,
    injection_data: Option<InjectionParseData>,
    text: Rope,
    folding: bool,
    parse_cancel: Arc<AtomicBool>,
    highlight_query: Option<Arc<tree_sitter::Query>>,
    visible: Option<Range<usize>>,
    previous: Option<Arc<super::super::highlighter::draw_first_cache::Cache>>,
    edits: Vec<InputEdit>,
    mut proof: super::super::highlighter::rust_empty::State,
    revision: u64,
) -> Option<(
    tree_sitter::Tree,
    Vec<InjectionLayer>,
    FoldCandidates,
    Option<Arc<super::super::highlighter::draw_first_cache::Cache>>,
)> {
    let (mut parser, grammar) = LanguageRegistry::singleton().parser(&language).ok()?;
    parser.set_language(&grammar).ok()?;
    let mut progress = |_: &tree_sitter::ParseState| {
        if parse_cancel.load(Ordering::Relaxed) {
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    };
    let options = ParseOptions::new().progress_callback(&mut progress);
    let tree = parser.parse_with_options(
        &mut |offset, _| parse_input_bytes(&text, offset),
        old_tree.as_ref(),
        Some(options),
    )?;
    if parse_cancel.load(Ordering::Relaxed) {
        return None;
    }
    let injections = injection_data.map_or_else(Default::default, |data| {
        let reuse = edits.len() == 1
            && proof.can_reuse(Some(&data.query), Some(&grammar))
            && !tree.root_node().has_error()
            && old_tree.as_ref().is_some_and(|old| {
                !old.root_node().has_error() && old.changed_ranges(&tree).next().is_none()
            });
        if reuse {
            proof.reused();
            Vec::new()
        } else {
            let (layers, zero) =
                SyntaxHighlighter::compute_injection_layers_observed(data, &tree, &text);
            proof.observed(zero);
            layers
        }
    });
    let fold_cache = if folding && highlight_query.is_some() {
        Some(Arc::new(super::draw_first_folds::Cache::update(
            previous.as_ref().and_then(|c| c.folds.as_deref()),
            old_tree.as_ref(),
            &tree,
            &text,
            &edits,
            revision,
        )))
    } else {
        None
    };
    let folds = fold_cache
        .as_ref()
        .map(|c| {
            if gpui_base::input::incremental_editor() {
                FoldCandidates::Cached(c.ranges.clone())
            } else {
                FoldCandidates::Owned(c.ranges.as_ref().clone())
            }
        })
        .unwrap_or_else(|| {
            FoldCandidates::Owned(if folding {
                extract_fold_ranges(&tree)
            } else {
                Vec::new()
            })
        });
    let cache = highlight_query.as_ref().map(|query| {
        let snapshot = super::super::highlighter::draw_first_cache::Snapshot {
            tree: Some(&tree),
            query: Some(query),
            text: &text,
            layers: &injections,
        };
        let mut built = if let Some(range) = visible.clone() {
            let unchanged = previous.is_some()
                && edits.iter().all(|e| {
                    e.start_byte >= range.end || e.new_end_byte.max(e.start_byte + 1) <= range.start
                })
                && old_tree.as_ref().is_some_and(|old| {
                    old.changed_ranges(&tree)
                        .all(|r| r.start_byte >= range.end || r.end_byte <= range.start)
                })
                && injections.is_empty();
            super::super::highlighter::draw_first_cache::Cache::build_scoped(
                snapshot,
                query.clone(),
                range,
                previous.as_deref(),
                &edits,
                unchanged,
            )
        } else {
            super::super::highlighter::draw_first_cache::Cache::build(snapshot)
        };
        built.proof = Some(proof.clone());
        built.folds = fold_cache.clone();
        Arc::new(built)
    });
    if parse_cancel.load(Ordering::Relaxed) {
        return None;
    }
    Some((tree, injections, folds, cache))
}

#[cfg(all(test, feature = "tree-sitter-rust"))]
mod tests {
    use super::*;
    use crate::highlighter::HighlightTheme;
    fn point(text: &str) -> Point {
        Point::new(
            text.bytes().filter(|b| *b == b'\n').count(),
            text.rsplit('\n').next().unwrap().len(),
        )
    }
    fn settle(h: &mut SyntaxHighlighter, next: &str) {
        let old = h.text().to_string();
        let text = Rope::from(next);
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: old.len(),
            new_end_byte: next.len(),
            start_position: Point::new(0, 0),
            old_end_position: point(&old),
            new_end_position: point(next),
        };
        h.edit_tree(Some(edit), &text);
        let (tree, layers, _, cache) = parse_scoped(
            h.language().clone(),
            h.tree().cloned(),
            h.injection_parse_data(),
            text.clone(),
            true,
            Arc::new(AtomicBool::new(false)),
            h.draw_first_query(),
            Some(0..next.len()),
            h.draw_first_pending().0,
            vec![edit],
            h.draw_first_proof(),
            h.draw_first_revision,
        )
        .unwrap();
        h.apply_background_tree(tree, &text, layers);
        h.replace_draw_first_cache(cache);
        let mut oracle = SyntaxHighlighter::new("rust");
        assert!(oracle.update(None, &text, None));
        let theme = HighlightTheme::default_dark();
        assert_eq!(h.text(), &text);
        assert_eq!(
            h.styles(&(0..next.len()), theme.as_ref()),
            oracle.styles(&(0..next.len()), theme.as_ref())
        );
        for start in next.char_indices().map(|(i, _)| i) {
            let range = start..next.len();
            assert_eq!(
                h.styles(&range, theme.as_ref()),
                oracle.styles(&range, theme.as_ref())
            );
        }
        assert_eq!(
            h.tree().unwrap().root_node().to_sexp(),
            oracle.tree().unwrap().root_node().to_sexp()
        );
    }
    #[test]
    fn insert_delete_paste_undo_and_ime_snapshots_equal_sync_colours() {
        let mut h = SyntaxHighlighter::new("rust");
        h.draw_first_cached = true;
        for next in [
            "fn main() {}",
            "fn main() { let a = 1; }",
            "fn main() { let a = ; }",
            "// pasted\nfn main() { println!(\"é\"); }",
            "fn main() { let a = 1; }",
            "fn main() { let café = 1; }",
            "fn main() { let か = 1; }",
            "fn main() { let 漢字 = 1; }",
        ] {
            settle(&mut h, next);
        }
    }
    #[test]
    fn scoped_worker_keeps_full_document_oracle_after_offscreen_multiline_edit() {
        let source = (0..10_000)
            .map(|i| format!("fn line_{i}() {{ let x = 1; }}\n"))
            .collect::<String>();
        let mut h = SyntaxHighlighter::new("rust");
        h.draw_first_cached = true;
        let text = Rope::from(source.as_str());
        h.edit_tree(None, &text);
        let (tree, layers, _, cache) = parse_scoped(
            "rust".into(),
            h.tree().cloned(),
            h.injection_parse_data(),
            text.clone(),
            false,
            Arc::new(AtomicBool::new(false)),
            h.draw_first_query(),
            Some(0..2048),
            None,
            Vec::new(),
            h.draw_first_proof(),
            h.draw_first_revision,
        )
        .unwrap();
        h.apply_background_tree(tree, &text, layers);
        h.replace_draw_first_cache(cache);
        let mut oracle = SyntaxHighlighter::new("rust");
        assert!(oracle.update(None, &text, None));
        let theme = HighlightTheme::default_dark();
        for range in [0..1024, 100_000..101_000, source.len() - 1024..source.len()] {
            assert_eq!(
                h.styles(&range, theme.as_ref()),
                oracle.styles(&range, theme.as_ref())
            );
        }
        let next = format!("/*{}", source);
        let text = Rope::from(next.as_str());
        h.edit_tree(None, &text);
        let (tree, layers, _, cache) = parse_scoped(
            "rust".into(),
            h.tree().cloned(),
            h.injection_parse_data(),
            text.clone(),
            false,
            Arc::new(AtomicBool::new(false)),
            h.draw_first_query(),
            Some(0..2048),
            None,
            Vec::new(),
            h.draw_first_proof(),
            h.draw_first_revision,
        )
        .unwrap();
        h.apply_background_tree(tree, &text, layers);
        h.replace_draw_first_cache(cache);
        assert!(oracle.update(None, &text, None));
        for range in [0..1024, 100_000..101_000, next.len() - 1024..next.len()] {
            assert_eq!(
                h.styles(&range, theme.as_ref()),
                oracle.styles(&range, theme.as_ref())
            );
        }
    }
    #[test]
    fn stale_worker_text_cannot_replace_later_edit() {
        let mut h = SyntaxHighlighter::new("rust");
        let old = Rope::from("fn old() {}");
        let new = Rope::from("fn new() {}");
        let (tree, layers, _, cache) = parse_snapshot(
            "rust".into(),
            None,
            None,
            old.clone(),
            false,
            Arc::new(AtomicBool::new(false)),
            None,
        )
        .unwrap();
        h.edit_tree(None, &new);
        h.apply_background_tree(tree, &old, layers);
        assert_eq!(h.text(), &new);
        assert!(h.tree().is_none());
    }
    #[test]
    fn cancelled_worker_does_not_apply() {
        assert!(
            parse_snapshot(
                "rust".into(),
                None,
                None,
                Rope::from("fn main() {}"),
                false,
                Arc::new(AtomicBool::new(true)),
                None
            )
            .is_none()
        );
    }
}
