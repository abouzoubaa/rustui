use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use gpui::{HighlightStyle, SharedString, Task};
use gpui_base::input::{
    EditorState, FoldRange, HighlightStyleResolver, InputEdit as BaseInputEdit, InputHighlighter,
    InputHighlighterFactory,
};
use ropey::Rope;
use tree_sitter::{InputEdit, ParseOptions, Point};

use super::{LanguageRegistry, SyntaxHighlighter, highlighter::parse_input_bytes};

pub(crate) fn input_highlighter_factory() -> InputHighlighterFactory {
    Rc::new(|language| {
        LanguageRegistry::singleton().has_parser(language).then(|| {
            Box::new(TreeSitterInputHighlighter::new(language)) as Box<dyn InputHighlighter>
        })
    })
}

#[path = "draw_first_background.rs"]
mod draw_first_background;
#[path = "draw_first_folds.rs"]
pub(crate) mod draw_first_folds;

const SYNC_PARSE_TIMEOUT: Duration = Duration::from_millis(2);
const SYNC_PARSE_MAX_BYTES: usize = 256 * 1024;
const PARSE_DEBOUNCE: Duration = Duration::from_millis(150);

struct TreeSitterInputHighlighter {
    inner: Rc<RefCell<SyntaxHighlighter>>,
    parse_task: Rc<RefCell<Option<Task<()>>>>,
    visible: Rc<RefCell<Range<usize>>>,
}

impl TreeSitterInputHighlighter {
    fn new(language: &str) -> Self {
        let mut inner = SyntaxHighlighter::new(language);
        inner.draw_first_cached = gpui::fast_draw_first_enabled();
        Self {
            inner: Rc::new(RefCell::new(inner)),
            parse_task: Rc::new(RefCell::new(None)),
            visible: Rc::new(RefCell::new(0..4096)),
        }
    }

    /// Settle after the edits reached the tree: drop a pending background
    /// parse when the synchronous one `completed`, or schedule one for `text`.
    fn finish_update(
        &mut self,
        completed: bool,
        text: &Rope,
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        if completed {
            self.parse_task.borrow_mut().take();
            return;
        }

        let highlighter = self.inner.clone();
        let parse_task = self.parse_task.clone();
        let language = highlighter.borrow().language().clone();
        let old_tree = highlighter.borrow().tree().cloned();
        let injection_data = highlighter.borrow().injection_parse_data();
        let highlight_query = highlighter.borrow().draw_first_query();
        let (previous, edits) = highlighter.borrow().draw_first_pending();
        let proof = highlighter.borrow().draw_first_proof();
        let revision = highlighter.borrow().draw_first_revision;
        let visible = self.visible.borrow().clone();
        let visible = visible.start.min(text.len())..visible.end.min(text.len());
        let visible_for_apply = visible.clone();
        let previous_for_apply = previous.clone();
        let edits_for_apply = edits.clone();
        let text = text.clone();
        let text_for_apply = text.clone();
        let cancel = Arc::new(AtomicBool::new(false));

        let task = cx.spawn_in(window, async move |entity, cx| {
            struct CancelOnDrop(Arc<AtomicBool>);
            impl Drop for CancelOnDrop {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::Relaxed);
                }
            }
            let _cancel_guard = CancelOnDrop(cancel.clone());
            if !gpui::fast_draw_first_enabled() {
                cx.background_executor().timer(PARSE_DEBOUNCE).await;
            }

            let parse_cancel = cancel.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    draw_first_background::parse_scoped(
                        language,
                        old_tree,
                        injection_data,
                        text,
                        folding,
                        parse_cancel,
                        highlight_query,
                        gpui::fast_draw_first_enabled().then_some(visible),
                        previous,
                        edits,
                        proof,
                        revision,
                    )
                })
                .await;

            if let Some((tree, injections, folds, cache)) = result {
                if gpui::fast_draw_first_enabled()
                    && (!highlighter.borrow().text().eq(&text_for_apply)
                        || highlighter.borrow().draw_first_revision != revision)
                {
                    return;
                }

                let fold_stats = cache
                    .as_ref()
                    .and_then(|cache| cache.folds.as_ref())
                    .map(|f| (f.visited, f.ranges.len()));
                highlighter
                    .borrow_mut()
                    .apply_background_tree(tree, &text_for_apply, injections);
                highlighter
                    .borrow_mut()
                    .replace_draw_first_cache(cache.clone());
                if !gpui::fast_draw_first_enabled() {
                    let _ = entity.update(cx, |state, cx| {
                        state.apply_highlighter_fold_candidates(folds, cx)
                    });
                    return;
                }
                let _ = entity.update_in(cx, |state, window, cx| {
                    if let Some((visited, ranges)) = fold_stats {
                        gpui::fast_latency_config(
                            window.window_handle().window_id(),
                            "g53_fold_cache",
                            [visited as f64, ranges as f64, 0., 0.],
                        );
                    }
                    let actual_visible = state
                        .highlighter_visible_range()
                        .unwrap_or_else(|| visible_for_apply.clone());
                    let before = previous_for_apply
                        .as_ref()
                        .map(|cache| cache.matches(actual_visible.clone(), &edits_for_apply));
                    let visible_changed = cache.as_ref().is_some_and(|cache| {
                        Some(cache.matches(actual_visible.clone(), &[])) != before
                    });
                    gpui::fast_latency_config(
                        window.window_handle().window_id(),
                        "g53_highlight_applied",
                        [text_for_apply.len() as f64, 0., 0., 0.],
                    );
                    gpui::fast_latency_config(
                        window.window_handle().window_id(),
                        "g53_colour_effect",
                        [
                            visible_changed as u8 as f64,
                            actual_visible.start as f64,
                            actual_visible.end as f64,
                            0.,
                        ],
                    );
                    if visible_changed {
                        gpui::fast_note_frame_cause(window.window_handle().window_id(), 4);
                    }
                    let _wake = gpui::fast_wake_cause(4);
                    state.apply_highlighter_fold_candidates_quiet(folds, visible_changed, cx);
                });
            }
        });
        parse_task.borrow_mut().replace(task);
    }
}

impl SyntaxHighlighter {
    pub(crate) fn update_input(
        &mut self,
        edit: Option<BaseInputEdit>,
        text: &Rope,
        timeout: Option<Duration>,
    ) -> bool {
        self.update(edit.map(to_tree_sitter_edit), text, timeout)
    }
}

impl InputHighlighter for TreeSitterInputHighlighter {
    fn language(&self) -> SharedString {
        self.inner.borrow().language().clone()
    }

    fn update(
        &mut self,
        edit: Option<BaseInputEdit>,
        text: &Rope,
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        let edit = edit.map(to_tree_sitter_edit);
        let completed = {
            let mut highlighter = self.inner.borrow_mut();
            if gpui::fast_draw_first_enabled() || text.len() > SYNC_PARSE_MAX_BYTES {
                highlighter.edit_tree(edit, text);
                false
            } else {
                highlighter.update(edit, text, Some(SYNC_PARSE_TIMEOUT))
            }
        };
        self.finish_update(completed, text, folding, window, cx);
    }

    fn update_batch(
        &mut self,
        edits: &[(BaseInputEdit, Rope)],
        folding: bool,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<EditorState>,
    ) {
        let Some((_, text)) = edits.last() else {
            return;
        };
        let edits: Vec<InputEdit> = edits
            .iter()
            .map(|(edit, _)| to_tree_sitter_edit(*edit))
            .collect();
        let completed = {
            let mut highlighter = self.inner.borrow_mut();
            if gpui::fast_draw_first_enabled() || text.len() > SYNC_PARSE_MAX_BYTES {
                for edit in edits {
                    highlighter.edit_tree(Some(edit), text);
                }
                false
            } else {
                highlighter.update_edits(&edits, text, Some(SYNC_PARSE_TIMEOUT))
            }
        };
        self.finish_update(completed, text, folding, window, cx);
    }

    fn styles(
        &self,
        range: &Range<usize>,
        resolver: &dyn HighlightStyleResolver,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut visible = self.visible.borrow_mut();
        // styles() is called per visible line; retain a bounded viewport around
        // the requested range rather than accumulating scrolled-away ranges.
        if range.start < visible.start || range.end > visible.end {
            *visible = range.start.saturating_sub(4096)..range.end.saturating_add(4096);
        }
        self.inner.borrow().styles(range, resolver)
    }
    fn fold_ranges(&self, _: &Rope) -> Vec<FoldRange> {
        self.inner
            .borrow()
            .tree()
            .map(extract_fold_ranges)
            .unwrap_or_default()
    }

    fn fold_ranges_for_edit(&self, range: Range<usize>, _: &Rope) -> Vec<FoldRange> {
        self.inner
            .borrow()
            .tree()
            .map(|tree| extract_fold_ranges_in_range(tree, range))
            .unwrap_or_default()
    }
}

fn to_tree_sitter_edit(edit: BaseInputEdit) -> InputEdit {
    InputEdit {
        start_byte: edit.start_byte,
        old_end_byte: edit.old_end_byte,
        new_end_byte: edit.new_end_byte,
        start_position: Point::new(edit.start_position.row, edit.start_position.column),
        old_end_position: Point::new(edit.old_end_position.row, edit.old_end_position.column),
        new_end_position: Point::new(edit.new_end_position.row, edit.new_end_position.column),
    }
}

fn extract_fold_ranges(tree: &tree_sitter::Tree) -> Vec<FoldRange> {
    extract_fold_ranges_in_range(tree, 0..usize::MAX)
}

fn extract_fold_ranges_in_range(
    tree: &tree_sitter::Tree,
    byte_range: Range<usize>,
) -> Vec<FoldRange> {
    fn collect(node: tree_sitter::Node, bytes: &Range<usize>, ranges: &mut Vec<FoldRange>) {
        if node.end_byte() <= bytes.start || node.start_byte() >= bytes.end {
            return;
        }
        let start = node.start_position().row;
        let end = node.end_position().row;
        if end.saturating_sub(start) < 2 {
            return;
        }
        ranges.push(FoldRange::new(start, end));
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            collect(child, bytes, ranges);
        }
    }

    let root = tree.root_node();
    let mut ranges = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        collect(child, &byte_range, &mut ranges);
    }
    ranges.sort_by_key(|range| range.start_line);
    ranges.dedup_by_key(|range| range.start_line);
    ranges
}
