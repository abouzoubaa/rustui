// RustUI G5-3b: snapshot parsing runs on GPUI's background executor.
use super::super::highlighter::{InjectionLayer, InjectionParseData};
use super::*;
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
        SyntaxHighlighter::compute_injection_layers(data, &tree, &text)
    });
    let folds = if folding {
        extract_fold_ranges(&tree)
    } else {
        Vec::new()
    };
    let cache = highlight_query.as_ref().map(|query| {
        Arc::new(super::super::highlighter::draw_first_cache::Cache::build(
            super::super::highlighter::draw_first_cache::Snapshot {
                tree: Some(&tree),
                query: Some(query),
                text: &text,
                layers: &injections,
            },
        ))
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
        let (tree, layers, _, cache) = parse_snapshot(
            h.language().clone(),
            h.tree().cloned(),
            h.injection_parse_data(),
            text.clone(),
            true,
            Arc::new(AtomicBool::new(false)),
            h.draw_first_query(),
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
