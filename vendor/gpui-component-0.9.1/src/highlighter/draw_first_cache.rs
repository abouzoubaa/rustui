// RustUI G5-3b: immutable capture snapshots; tree-sitter queries run in the worker.
use super::*;
pub(crate) struct Snapshot<'a> {
    pub tree: Option<&'a Tree>,
    pub query: Option<&'a Query>,
    pub text: &'a Rope,
    pub layers: &'a [InjectionLayer],
}
impl Snapshot<'_> {
    pub(super) fn matches(&self, range: Range<usize>) -> Vec<HighlightItem> {
        let mut highlights = vec![];
        let mut injection_highlights = vec![];
        let Some(tree) = &self.tree else {
            return highlights;
        };

        let Some(query) = &self.query else {
            return highlights;
        };

        let root_node = tree.root_node();
        let source = &self.text;

        // Query pre-parsed injection layers.
        let mut last_layer_start = 0;
        for layer in self.layers {
            debug_assert!(layer.byte_range.start >= last_layer_start);
            last_layer_start = layer.byte_range.start;

            if layer.byte_range.end <= range.start {
                continue;
            }

            // Layers are sorted by start byte in compute_injection_layers.
            if layer.byte_range.start >= range.end {
                break;
            }

            let query = &layer.highlight_query;

            let mut query_cursor = QueryCursor::new();
            query_cursor.set_byte_range(range.clone());

            let mut matches =
                query_cursor.matches(query, layer.tree.root_node(), TextProvider(&self.text));

            let mut last_end = 0usize;
            while let Some(m) = matches.next() {
                let allow_overlapping_captures = query
                    .property_settings(m.pattern_index)
                    .iter()
                    .any(|prop| prop.key.as_ref() == "highlight.allow-overlap");

                for cap in m.captures() {
                    let node_range = cap.node.start_byte()..cap.node.end_byte();

                    if !allow_overlapping_captures && node_range.start < last_end {
                        continue;
                    }

                    if let Some(highlight_name) = query.capture_names().get(cap.index as usize) {
                        if !allow_overlapping_captures {
                            last_end = node_range.end;
                        }
                        injection_highlights.push(HighlightItem::new(
                            node_range,
                            SharedString::from(highlight_name.to_string()),
                        ));
                    }
                }
            }
        }

        let mut query_cursor = QueryCursor::new();
        query_cursor.set_byte_range(range.clone());

        let mut matches = query_cursor.matches(query, root_node, TextProvider(source));

        while let Some(query_match) = matches.next() {
            for cap in query_match.captures() {
                let node = cap.node;

                let Some(highlight_name) = query.capture_names().get(cap.index as usize) else {
                    continue;
                };

                let node_range: Range<usize> = node.start_byte()..node.end_byte();
                let highlight_name = SharedString::from(highlight_name.to_string());

                // Merge near range and same highlight name
                let last_item = highlights.last();
                let last_range = last_item.map(|item| &item.range).unwrap_or(&(0..0));
                let last_highlight_name = last_item.map(|item| item.name.clone());

                if last_range == &node_range {
                    // case:
                    // last_range: 213..220, last_highlight_name: Some("property")
                    // last_range: 213..220, last_highlight_name: Some("string")
                    highlights.push(HighlightItem::new(
                        node_range,
                        last_highlight_name.unwrap_or(highlight_name),
                    ));
                } else {
                    highlights.push(HighlightItem::new(node_range, highlight_name.clone()));
                }
            }
        }

        // Injected languages are more specific than the host language. Keep
        // them last so their colors win over broad Markdown captures such as
        // `fenced_code_block @text.literal`.
        highlights.extend(injection_highlights);

        // DO NOT REMOVE THIS PRINT, it's useful for debugging
        // for item in highlights {
        //     println!("item: {:?}", item);
        // }

        highlights
    }
}
/// Preserve query precedence while indexing byte buckets; no document walk on paint.
pub(crate) struct Cache {
    tokens: Vec<HighlightItem>,
    buckets: Vec<Vec<usize>>,
    covered: Range<usize>,
    edited: Vec<(Range<usize>, Vec<HighlightItem>)>,
    fallback: Option<OwnedSnapshot>,
    pub(crate) proof: Option<super::rust_empty::State>,
}
struct OwnedSnapshot {
    tree: Option<Tree>,
    query: Option<Arc<Query>>,
    text: Rope,
    layers: Vec<InjectionLayer>,
}
impl OwnedSnapshot {
    fn matches(&self, range: Range<usize>) -> Vec<HighlightItem> {
        Snapshot {
            tree: self.tree.as_ref(),
            query: self.query.as_deref(),
            text: &self.text,
            layers: &self.layers,
        }
        .matches(range)
    }
}
impl Cache {
    pub(crate) fn build_scoped(
        snapshot: Snapshot<'_>,
        query: Arc<Query>,
        range: Range<usize>,
        previous: Option<&Cache>,
        edits: &[InputEdit],
        unchanged: bool,
    ) -> Self {
        let len = snapshot.text.len();
        let range = range.start.min(len)..range.end.min(len);
        let tokens = if unchanged {
            previous
                .map(|c| c.matches(range.clone(), edits))
                .unwrap_or_else(|| snapshot.matches(range.clone()))
        } else {
            snapshot.matches(range.clone())
        };
        let fallback = OwnedSnapshot {
            tree: snapshot.tree.cloned(),
            query: Some(query),
            text: snapshot.text.clone(),
            layers: snapshot.layers.to_vec(),
        };
        let mut edited = Vec::new();
        for edit in edits {
            let start = snapshot.text.offset_to_point(edit.start_byte.min(len)).row;
            let end = snapshot
                .text
                .offset_to_point(edit.new_end_byte.min(len))
                .row;
            let touched =
                snapshot.text.line_start_offset(start)..snapshot.text.line_end_offset(end);
            if touched.start >= range.start && touched.end <= range.end {
                continue;
            }
            if edited
                .iter()
                .any(|(r, _): &(Range<usize>, Vec<HighlightItem>)| r == &touched)
            {
                continue;
            }
            edited.push((touched.clone(), snapshot.matches(touched)));
        }
        let mut result = Self::indexed(tokens, len, range, Some(fallback));
        result.edited = edited;
        result
    }
    fn indexed(
        tokens: Vec<HighlightItem>,
        len: usize,
        covered: Range<usize>,
        fallback: Option<OwnedSnapshot>,
    ) -> Self {
        let mut buckets = vec![Vec::new(); len / 1024 + 1];
        for (id, t) in tokens.iter().enumerate() {
            if t.range.is_empty() {
                continue;
            }
            let end = (t.range.end.saturating_sub(1) / 1024).min(buckets.len() - 1);
            for bucket in &mut buckets[t.range.start.min(len) / 1024..=end] {
                bucket.push(id);
            }
        }
        Self {
            tokens,
            buckets,
            covered,
            edited: Vec::new(),
            fallback,
            proof: None,
        }
    }
    pub(crate) fn build(snapshot: Snapshot<'_>) -> Self {
        let len = snapshot.text.len();
        let tokens = snapshot.matches(0..len);
        let mut buckets = vec![Vec::new(); len / 1024 + 1];
        for (id, t) in tokens.iter().enumerate() {
            if t.range.is_empty() {
                continue;
            }
            let end = (t.range.end.saturating_sub(1) / 1024).min(buckets.len() - 1);
            for bucket in &mut buckets[t.range.start.min(len) / 1024..=end] {
                bucket.push(id);
            }
        }
        Self {
            tokens,
            buckets,
            covered: 0..len,
            edited: Vec::new(),
            fallback: None,
            proof: None,
        }
    }
    fn base_matches(&self, range: Range<usize>) -> Vec<HighlightItem> {
        if range.is_empty() || range.start / 1024 >= self.buckets.len() {
            return Vec::new();
        }
        if (range.start < self.covered.start || range.end > self.covered.end)
            && let Some(snapshot) = &self.fallback
        {
            // Newly exposed text is queried against the current immutable tree;
            // stale offscreen spans are never used as an oracle shortcut.
            if let Some((_, tokens)) = self
                .edited
                .iter()
                .find(|(r, _)| range.start >= r.start && range.end <= r.end)
            {
                return tokens
                    .iter()
                    .filter(|t| t.range.start < range.end && t.range.end > range.start)
                    .cloned()
                    .collect();
            }
            return snapshot.matches(range);
        }
        let mut ids = Vec::new();
        let end = (range.end.saturating_sub(1) / 1024).min(self.buckets.len() - 1);
        for bucket in &self.buckets[range.start / 1024..=end] {
            ids.extend_from_slice(bucket);
        }
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .filter_map(|i| {
                let t = &self.tokens[i];
                (t.range.start < range.end && t.range.end > range.start).then(|| t.clone())
            })
            .collect()
    }
}

impl Cache {
    pub(crate) fn matches(&self, range: Range<usize>, edits: &[InputEdit]) -> Vec<HighlightItem> {
        let mut original = range.clone();
        for e in edits.iter().rev() {
            let back = |p: usize, upper: bool| {
                if p < e.start_byte {
                    p
                } else if p > e.new_end_byte {
                    e.old_end_byte + p - e.new_end_byte
                } else if upper {
                    e.old_end_byte
                } else {
                    e.start_byte
                }
            };
            original = back(original.start, false).saturating_sub(1)
                ..back(original.end, true).saturating_add(1);
        }
        self.base_matches(original)
            .into_iter()
            .filter_map(|mut token| {
                for e in edits {
                    let p = token.range.start;
                    let start = if p >= e.old_end_byte {
                        e.new_end_byte + p - e.old_end_byte
                    } else if p >= e.start_byte {
                        e.new_end_byte
                    } else {
                        p
                    };
                    let p = token.range.end;
                    let end = if p > e.old_end_byte {
                        e.new_end_byte + p - e.old_end_byte
                    } else if p > e.start_byte {
                        e.start_byte
                    } else {
                        p
                    };
                    token.range = start..end;
                }
                (!token.range.is_empty()
                    && token.range.start < range.end
                    && token.range.end > range.start)
                    .then_some(token)
            })
            .collect()
    }
}
impl SyntaxHighlighter {
    pub(crate) fn replace_draw_first_cache(&mut self, cache: Option<Arc<Cache>>) {
        if let Some(proof) = cache.as_ref().and_then(|cache| cache.proof.clone()) {
            self.rust_empty = proof;
        }
        self.draw_first_cache = cache;
        self.draw_first_edits.clear();
    }
}
#[cfg(test)]
mod edit_tests {
    use super::*;
    fn e(start: usize, old: usize, new: usize) -> InputEdit {
        InputEdit {
            start_byte: start,
            old_end_byte: old,
            new_end_byte: new,
            start_position: Point::new(0, start),
            old_end_position: Point::new(0, old),
            new_end_position: Point::new(0, new),
        }
    }
    fn cache() -> Cache {
        Cache {
            tokens: vec![HighlightItem::new(0..2, "a"), HighlightItem::new(4..6, "b")],
            buckets: vec![vec![0, 1]],
            covered: 0..6,
            edited: Vec::new(),
            fallback: None,
            proof: None,
        }
    }
    #[test]
    fn insertion_shifts_existing_spans_without_rewriting_cache() {
        let c = cache();
        let spans = c.matches(0..8, &[e(3, 3, 5)]);
        assert_eq!(
            spans.iter().map(|s| s.range.clone()).collect::<Vec<_>>(),
            vec![0..2, 6..8]
        );
        assert_eq!(c.tokens[1].range, 4..6);
    }
    #[test]
    fn deletion_clips_a_crossing_span_and_shifts_the_following_span() {
        let c = cache();
        let spans = c.matches(0..4, &[e(1, 3, 1)]);
        assert_eq!(
            spans.iter().map(|s| s.range.clone()).collect::<Vec<_>>(),
            vec![0..1, 2..4]
        );
    }
}
