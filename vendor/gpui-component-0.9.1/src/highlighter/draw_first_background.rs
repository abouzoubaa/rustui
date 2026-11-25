// RustUI G5-3b: snapshot parsing runs on GPUI's background executor.
use super::super::highlighter::{InjectionLayer, InjectionParseData};
use super::*;
#[cfg(test)]
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
    Vec<FoldRange>,
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
    Vec<FoldRange>,
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
    use super::super::highlighter::rust_injection_ranges::{self, Run, Trace};
    proof.trace = None;
    let injections = injection_data.map_or_else(Default::default, |data| {
        let trace_enabled = rust_injection_ranges::trace_enabled() || cfg!(test);
        let started = trace_enabled.then(std::time::Instant::now);
        let scoped_query = highlight_query.as_ref().filter(|_| {
            visible.is_some()
                && rust_injection_ranges::enabled()
                && proof.current(Some(&data.query), Some(&grammar))
        });
        let scoped_old = scoped_query.and(old_tree.as_ref()).map(|old| {
            let changes = old
                .changed_ranges(&tree)
                .map(|range| range.start_byte..range.end_byte)
                .collect::<Vec<_>>();
            (old, changes)
        });
        let has_structural_changes = std::cell::OnceCell::new();
        let structural_changes = || {
            *has_structural_changes.get_or_init(|| {
                scoped_old.as_ref().map_or_else(
                    || {
                        old_tree
                            .as_ref()
                            .is_some_and(|old| old.changed_ranges(&tree).next().is_some())
                    },
                    |(_, ranges)| !ranges.is_empty(),
                )
            })
        };
        let failures = trace_enabled.then(|| {
            proof.failed_predicates(
                &data.query,
                &grammar,
                edits.len(),
                old_tree.as_ref(),
                &tree,
                structural_changes(),
            )
        });
        let reusable_empty = (scoped_query.is_none()
            || proof
                .captures
                .as_ref()
                .is_some_and(|index| index.is_empty()))
            && edits.len() == 1
            && proof.can_reuse(Some(&data.query), Some(&grammar))
            && !tree.root_node().has_error()
            && old_tree
                .as_ref()
                .is_some_and(|old| !old.root_node().has_error())
            && !structural_changes();
        let (layers, run) = if let Some(query) = scoped_query {
            if reusable_empty {
                proof.reused();
                (Vec::new(), Run::ScopedEmptyReuse)
            } else {
                rust_injection_ranges::update(
                    data, scoped_old, &tree, &text, &edits, &mut proof, query, revision,
                )
            }
        } else {
            proof.captures = None;
            if reusable_empty {
                proof.reused();
                (Vec::new(), Run::BaselineReuse)
            } else {
                let (layers, zero) =
                    SyntaxHighlighter::compute_injection_layers_observed(data, &tree, &text);
                proof.observed(zero);
                (layers, Run::BaselineFull)
            }
        };
        if let Some((failures, started)) = failures.zip(started) {
            proof.trace = Some(Trace {
                failures,
                run,
                query_ms: started.elapsed().as_secs_f64() * 1000.0,
                worker_thread: rust_injection_ranges::worker_thread_id(),
            });
        }
        layers
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
        .map(|c| c.ranges.as_ref().clone())
        .unwrap_or_else(|| {
            if folding {
                extract_fold_ranges(&tree)
            } else {
                Vec::new()
            }
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

#[cfg(all(test, feature = "tree-sitter-rust"))]
mod range_injection_oracle {
    use super::*;
    use crate::highlighter::HighlightTheme;
    use crate::highlighter::highlighter::rust_injection_ranges::Run;

    fn point(text: &str) -> Point {
        Point::new(
            text.bytes().filter(|&b| b == b'\n').count(),
            text.rsplit('\n').next().unwrap().len(),
        )
    }

    fn compare(h: &SyntaxHighlighter, text: &Rope) {
        let mut full = SyntaxHighlighter::new("rust");
        assert!(full.update(None, text, None));
        let layers = |v: &SyntaxHighlighter| {
            v.injection_parse_data()
                .unwrap()
                .old_layers
                .into_iter()
                .map(|layer| {
                    (
                        layer.language_name,
                        layer.ranges,
                        layer.tree.root_node().to_sexp(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let actual = layers(h);
        let expected = layers(&full);
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(actual, expected);
        }
        let theme = HighlightTheme::default_dark();
        assert_eq!(
            h.styles(&(0..text.len()), theme.as_ref()),
            full.styles(&(0..text.len()), theme.as_ref())
        );
    }

    fn settle(h: &mut SyntaxHighlighter, text: &Rope, expected: Run) {
        let mut proof = h.draw_first_proof();
        proof.enabled = true;
        let (previous, edits) = h.draw_first_pending();
        let (tree, layers, _, cache) = parse_scoped(
            "rust".into(),
            h.tree().cloned(),
            h.injection_parse_data(),
            text.clone(),
            true,
            Arc::new(AtomicBool::new(false)),
            h.draw_first_query(),
            Some(0..text.len()),
            previous,
            edits,
            proof,
            h.draw_first_revision,
        )
        .unwrap();
        assert_eq!(
            cache
                .as_ref()
                .unwrap()
                .proof
                .as_ref()
                .unwrap()
                .trace
                .unwrap()
                .run,
            expected
        );
        h.apply_background_tree(tree, text, layers);
        h.replace_draw_first_cache(cache);
        compare(h, text);
    }

    fn edit(
        h: &mut SyntaxHighlighter,
        source: &mut String,
        offset: usize,
        old: usize,
        insert: &str,
    ) {
        let end = offset + old;
        let start_position = point(&source[..offset]);
        let old_end_position = point(&source[..end]);
        source.replace_range(offset..end, insert);
        let new_end_byte = offset + insert.len();
        h.edit_tree(
            Some(InputEdit {
                start_byte: offset,
                old_end_byte: end,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position: point(&source[..new_end_byte]),
            }),
            &Rope::from(source.as_str()),
        );
    }

    #[test]
    fn scoped_injections_and_styles_equal_full_recomputation() {
        let mut h = SyntaxHighlighter::new("rust");
        h.draw_first_cached = true;
        let mut source = "// header\nfn main() { example!(if true { 10 }); untouched!((20)); }\nmacro_rules! other { () => { 3 }; }\n".to_owned();
        let text = Rope::from(source.as_str());
        h.edit_tree(None, &text);
        settle(&mut h, &text, Run::LoadFull);

        // Outside edits, inside token-tree, new macro, removed macro, and
        // invalid EOF input all use the range path. Reversing each edit
        // also compares the shifted layers with a fresh document.
        for (needle, replacement) in [
            ("header", "larger header"),
            ("10", "30 + 40"),
            ("20", "\n25"),
            ("example!", "example"),
            ("10", "inner!(42)"),
            ("// header", "// header\nfn extra() { paste!((5)); }"),
            ("", "x"),
        ] {
            let at = if needle.is_empty() {
                source.len()
            } else {
                source.find(needle).unwrap()
            };
            edit(&mut h, &mut source, at, needle.len(), replacement);
            settle(&mut h, &Rope::from(source.as_str()), Run::Scoped);
            edit(&mut h, &mut source, at, replacement.len(), needle);
            settle(&mut h, &Rope::from(source.as_str()), Run::Scoped);
        }

        // A worker can be cancelled between edits. Coordinates must advance
        // through both edits since the last completed snapshot.
        edit(&mut h, &mut source, 0, 0, "// inserted\n");
        let inside = source.find("10").unwrap();
        edit(&mut h, &mut source, inside, 2, "99");
        settle(&mut h, &Rope::from(source.as_str()), Run::Scoped);
    }

    #[test]
    fn scoped_injections_reuse_the_original_px3_empty_proof() {
        let mut h = SyntaxHighlighter::new("rust");
        h.draw_first_cached = true;
        let mut source = "// header\nfn example() {}\n".to_owned();
        let text = Rope::from(source.as_str());
        h.edit_tree(None, &text);
        settle(&mut h, &text, Run::LoadFull);
        let comment = source.find("header").unwrap();
        edit(&mut h, &mut source, comment, "header".len(), "different");
        settle(&mut h, &Rope::from(source.as_str()), Run::ScopedEmptyReuse);
        let at = source.find(" {}").unwrap();
        edit(&mut h, &mut source, at, 3, " { nested!(42); }");
        settle(&mut h, &Rope::from(source.as_str()), Run::Scoped);
    }

    #[test]
    fn scoped_injections_preserve_full_query_caps_when_a_slot_opens() {
        let mut h = SyntaxHighlighter::new("rust");
        h.draw_first_cached = true;
        let mut source = "fn main() {\n".to_owned();
        for _ in 0..520 {
            source.push_str("emit!{fn example() {}};\n");
        }
        source.push('}');
        let text = Rope::from(source.as_str());
        h.edit_tree(None, &text);
        settle(&mut h, &text, Run::LoadFull);
        let at = source.find("emit!").unwrap();
        edit(
            &mut h,
            &mut source,
            at,
            "emit!{fn example() {}};\n".len(),
            "",
        );
        settle(&mut h, &Rope::from(source.as_str()), Run::Scoped);
    }
}
