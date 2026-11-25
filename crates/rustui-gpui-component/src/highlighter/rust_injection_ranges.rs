// G5-L9: scoped queries for PX-3's pinned two Rust token-tree patterns.
use super::*;
use std::ops::Range as ByteRange;
use tree_sitter::Range as NodeRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Capture {
    // The matcher is rooted at the macro, not at the injected token tree.
    scope: NodeRange,
    content: NodeRange,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Run {
    BaselineReuse = 0,
    BaselineFull = 1,
    LoadFull = 2,
    UnavailableFull = 4,
    Scoped = 5,
    IncompleteFull = 6,
    MissingTreeFull = 7,
    ScopedEmptyReuse = 8,
}

#[derive(Clone, Copy)]
pub(crate) struct Trace {
    pub(crate) failures: u16,
    pub(crate) run: Run,
    pub(crate) query_ms: f64,
    pub(crate) worker_thread: u64,
}

pub(crate) fn trace_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_G5_L9_LOG").as_deref() == Ok("1"))
}

pub(crate) fn worker_thread_id() -> u64 {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn pthread_threadid_np(thread: *mut std::ffi::c_void, id: *mut u64) -> std::ffi::c_int;
        }
        let mut id = 0;
        // A null pthread requests the caller's kernel thread ID.
        if unsafe { pthread_threadid_np(std::ptr::null_mut(), &mut id) } == 0 {
            return id;
        }
    }
    0
}

pub(crate) fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    // Only the existing RUSTUI_HL_FASTPATH opt-in reaches this gate.
    cfg!(test) || *ON.get_or_init(|| std::env::var("RUSTUI_G5_L9_SCOPED").as_deref() == Ok("1"))
}

fn bytes(range: NodeRange) -> ByteRange<usize> {
    range.start_byte..range.end_byte
}

fn intersects(a: ByteRange<usize>, b: &ByteRange<usize>) -> bool {
    a.start < a.end && a.start < b.end && b.start < a.end
}

fn affected_ranges(
    old: &Tree,
    tree: &Tree,
    structural: Vec<ByteRange<usize>>,
    edits: &[InputEdit],
    len: usize,
) -> Vec<ByteRange<usize>> {
    let mut ranges = structural;
    // Structural ranges omit some text edits. The one-byte halo also
    // discovers zero-width deletions and changes at node boundaries.
    for (i, edit) in edits.iter().enumerate() {
        let mut range = NodeRange {
            start_byte: edit.start_byte,
            end_byte: edit.new_end_byte,
            start_point: edit.start_position,
            end_point: edit.new_end_position,
        };
        for subsequent in &edits[i + 1..] {
            subsequent.edit_range(&mut range);
        }
        ranges.push(
            range.start_byte.saturating_sub(1).min(len)..range.end_byte.saturating_add(1).min(len),
        );
    }
    // Damage may touch a macro name rather than the captured token tree.
    // Include the enclosing match root at either edge on both trees.
    let affected = ranges.clone();
    for range in affected.iter().filter(|r| !r.is_empty()) {
        for syntax in [old, tree] {
            for byte in [range.start, range.end - 1] {
                let mut node = syntax
                    .root_node()
                    .descendant_for_byte_range(byte, byte.saturating_add(1));
                while let Some(current) = node {
                    if matches!(current.kind(), "macro_invocation" | "macro_rule") {
                        ranges.push(current.byte_range());
                    }
                    node = current.parent();
                }
            }
        }
    }
    ranges.sort_unstable_by_key(|r| r.start);
    let mut merged: Vec<ByteRange<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges.into_iter().filter(|r| !r.is_empty()) {
        if let Some(last) = merged.last_mut().filter(|last| last.end >= range.start) {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    merged
}

fn collect(
    data: &InjectionParseData,
    tree: &Tree,
    text: &Rope,
    scope: Option<ByteRange<usize>>,
    captures: &mut Vec<Capture>,
) -> bool {
    let mut cursor = QueryCursor::new();
    if let Some(scope) = &scope {
        cursor.set_byte_range(scope.clone());
    }
    let mut matches = cursor.matches(&data.query, tree.root_node(), TextProvider(text));
    while let Some(m) = matches.next() {
        for cap in m
            .captures()
            .iter()
            .filter(|cap| Some(cap.index) == data.content_capture_index)
        {
            let Some(root) = cap.node.parent() else {
                return false;
            };
            let capture = Capture {
                scope: root.range(),
                content: cap.node.range(),
            };
            if scope
                .as_ref()
                .is_none_or(|dirty| intersects(bytes(capture.scope), dirty))
            {
                captures.push(capture);
            }
        }
    }
    drop(matches);
    !cursor.did_exceed_match_limit()
}

fn materialize(
    old_layers: Vec<ReusableInjectionLayer>,
    captures: &[Capture],
    dirty: &[ByteRange<usize>],
    full: bool,
    text: &Rope,
    highlight_query: &Arc<Query>,
) -> Vec<InjectionLayer> {
    let mut old: HashMap<_, _> = old_layers
        .into_iter()
        .filter_map(|layer| {
            let range = *layer.ranges.first()?;
            Some(((range.start_byte, range.end_byte), layer))
        })
        .collect();
    let language = SharedString::from("rust");
    let mut layers = Vec::new();
    // The pinned patterns have one token tree per rooted match and no
    // combined layers. Keep the full path's range and per-document limits.
    for cap in captures
        .iter()
        .filter(|cap| injection_ranges_within_limits(std::slice::from_ref(&cap.content)))
        .take(MAX_NON_COMBINED_INJECTION_PARSES)
    {
        let capture = cap.content;
        let old = old.remove(&(capture.start_byte, capture.end_byte));
        if !full
            && !dirty.iter().any(|r| intersects(bytes(cap.scope), r))
            && old.as_ref().is_some_and(|layer| layer.ranges == [capture])
        {
            let layer = old.unwrap();
            layers.push(InjectionLayer {
                language_name: layer.language_name,
                highlight_query: layer.highlight_query,
                ranges: layer.ranges,
                byte_range: bytes(capture),
                tree: layer.tree,
                combined: false,
            });
            continue;
        }
        if let Some(layer) = SyntaxHighlighter::parse_injection_layer(
            &language,
            highlight_query.clone(),
            vec![capture],
            old.as_ref().map(|l| &l.tree),
            false,
            text,
        ) {
            layers.push(layer);
        }
    }
    layers
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    data: InjectionParseData,
    old: Option<(&Tree, Vec<ByteRange<usize>>)>,
    tree: &Tree,
    text: &Rope,
    edits: &[InputEdit],
    proof: &mut rust_empty::State,
    highlight_query: &Arc<Query>,
    revision: u64,
) -> (Vec<InjectionLayer>, Run) {
    let mut run = if old.is_none() {
        if revision <= 1 {
            Run::LoadFull
        } else {
            Run::MissingTreeFull
        }
    } else if proof.captures.is_some() && !edits.is_empty() {
        Run::Scoped
    } else {
        Run::UnavailableFull
    };
    let mut captures = Vec::new();
    let mut dirty = Vec::new();
    if let (Run::Scoped, Some((old, structural)), Some(saved)) = (run, old, &proof.captures) {
        dirty = affected_ranges(old, tree, structural, edits, text.len());
        captures.extend(saved.iter().copied().filter_map(|mut entry| {
            for edit in edits {
                edit.edit_range(&mut entry.scope);
                edit.edit_range(&mut entry.content);
            }
            (!bytes(entry.scope).is_empty()
                && !dirty.iter().any(|r| intersects(bytes(entry.scope), r)))
            .then_some(entry)
        }));
        if !dirty
            .iter()
            .all(|range| collect(&data, tree, text, Some(range.clone()), &mut captures))
        {
            run = Run::IncompleteFull;
            captures.clear();
        }
    }
    let full = !matches!(run, Run::Scoped);
    if full && !collect(&data, tree, text, None, &mut captures) {
        proof.captures = None;
        let (layers, zero) = SyntaxHighlighter::compute_injection_layers_observed(data, tree, text);
        proof.observed(zero);
        return (layers, run);
    }
    captures.sort_by_key(|entry| (entry.content.start_byte, entry.content.end_byte));
    captures.dedup();
    let layers = materialize(
        data.old_layers,
        &captures,
        &dirty,
        full,
        text,
        highlight_query,
    );
    proof.observed(captures.is_empty());
    proof.captures = Some(captures.into());
    (layers, run)
}
