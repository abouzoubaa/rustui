// RustUI G5-3d: fold results belong to a parsed tree, never to a paint.
use super::{FoldRange, InputEdit, Range};
use gpui_base::input::RopeExt as _;
use ropey::Rope;
use std::sync::Arc;
use tree_sitter::{Node, Tree};
#[derive(Clone, Debug)]
struct Record {
    start: usize,
    end: usize,
}
pub(crate) struct Cache {
    // Explicit edit revision; cloned Tree node addresses are not identities.
    revision: u64,
    records: Arc<Vec<Record>>,
    pub(crate) ranges: Arc<Vec<FoldRange>>,
    pub(crate) visited: usize,
}
fn touch(a: &Range<usize>, b: &Range<usize>) -> bool {
    if a.is_empty() {
        b.start <= a.start && a.start <= b.end
    } else if b.is_empty() {
        a.start <= b.start && b.start <= a.end
    } else {
        a.start < b.end && b.start < a.end
    }
}
fn collect(node: Node<'_>, out: &mut Vec<Record>, visited: &mut usize) {
    *visited += 1;
    if node
        .end_position()
        .row
        .saturating_sub(node.start_position().row)
        < 2
    {
        return;
    }
    out.push(Record {
        start: node.start_byte(),
        end: node.end_byte(),
    });
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect(child, out, visited);
    }
}
fn map_point(p: usize, e: &InputEdit, start: bool) -> Option<usize> {
    if p < e.start_byte || (!start && p == e.start_byte) {
        Some(p)
    } else if p >= e.old_end_byte {
        Some(e.new_end_byte + p - e.old_end_byte)
    } else {
        None
    }
}
impl Cache {
    pub(crate) fn update(
        previous: Option<&Self>,
        old: Option<&Tree>,
        tree: &Tree,
        text: &Rope,
        edits: &[InputEdit],
        revision: u64,
    ) -> Self {
        if let Some(p) = previous
            && p.revision == revision
            && edits.is_empty()
        {
            return Self {
                revision,
                records: p.records.clone(),
                ranges: p.ranges.clone(),
                visited: 0,
            };
        }
        let incremental = previous
            .is_some_and(|p| p.revision.wrapping_add(edits.len() as u64) == revision)
            && old.is_some()
            && !edits.is_empty();
        let mut records = Vec::new();
        let mut regions = Vec::new();
        let mut visited = 0;
        if incremental {
            for r in previous.unwrap().records.iter() {
                let mut mapped = Some(r.clone());
                for e in edits {
                    mapped = mapped.and_then(|r| {
                        Some(Record {
                            start: map_point(r.start, e, true)?,
                            end: map_point(r.end, e, false)?,
                        })
                    });
                }
                if let Some(r) = mapped
                    && r.start <= r.end
                    && r.end <= text.len()
                {
                    records.push(r);
                }
            }
            regions.extend(
                old.unwrap()
                    .changed_ranges(tree)
                    .map(|r| r.start_byte..r.end_byte),
            );
            // Covered endpoints need local rebuilding even if node kinds did
            // not change (e.g. a pasted block with the same AST shape).
            let covered = previous.unwrap().records.iter().any(|r| {
                edits.iter().any(|e| {
                    (r.start >= e.start_byte && r.start < e.old_end_byte)
                        || (r.end > e.start_byte && r.end < e.old_end_byte)
                })
            });
            let row_edit = edits.iter().any(|e| {
                e.start_position.row != e.old_end_position.row
                    || e.start_position.row != e.new_end_position.row
            });
            if !covered
                && !row_edit
                && regions.is_empty()
                && records.len() == previous.unwrap().records.len()
            {
                // Node structure and every line coordinate are unchanged.
                // Keep the exact fold vector; only byte anchors move for a
                // later structural edit. No row lookups, sorting or new ranges.
                return Self {
                    revision,
                    records: Arc::new(records),
                    ranges: previous.unwrap().ranges.clone(),
                    visited: 0,
                };
            }
            if records.len() != previous.unwrap().records.len() {
                regions.push(0..text.len());
            }
            if covered || row_edit || !regions.is_empty() {
                let growth = edits
                    .iter()
                    .map(|e| e.new_end_byte.saturating_sub(e.start_byte))
                    .sum::<usize>();
                let shrink = edits
                    .iter()
                    .map(|e| e.old_end_byte.saturating_sub(e.start_byte))
                    .sum::<usize>();
                let lo = edits
                    .iter()
                    .map(|e| e.start_byte)
                    .min()
                    .unwrap()
                    .saturating_sub(shrink);
                let hi = edits
                    .iter()
                    .map(|e| e.new_end_byte)
                    .max()
                    .unwrap()
                    .saturating_add(growth)
                    .saturating_add(1)
                    .min(text.len());
                regions.push(lo..hi);
            }
        }
        if !incremental {
            regions.push(0..text.len());
        }
        if !regions.is_empty() {
            // Rebuild complete affected top-level named subtrees, preserving
            // ancestor folds and all nested folds; unrelated subtrees stay cached.
            let mut cursor = tree.walk();
            let mut expanded = Vec::new();
            let mut fresh = Vec::new();
            for child in tree.root_node().named_children(&mut cursor) {
                let area = child.start_byte()..child.end_byte();
                if regions.iter().any(|r| touch(r, &area)) {
                    expanded.push(area);
                    collect(child, &mut fresh, &mut visited);
                }
            }
            records.retain(|r| !expanded.iter().any(|a| touch(a, &(r.start..r.end))));
            records.extend(fresh);
        }
        records.sort_by_key(|r| {
            (
                text.offset_to_point(r.start).row,
                r.start,
                std::cmp::Reverse(r.end),
            )
        });
        let mut ranges = records
            .iter()
            .filter_map(|r| {
                let start = text.offset_to_point(r.start).row;
                let end = text.offset_to_point(r.end).row;
                (end.saturating_sub(start) >= 2).then(|| FoldRange::new(start, end))
            })
            .collect::<Vec<_>>();
        ranges.dedup_by_key(|r| r.start_line);
        Self {
            revision,
            records: Arc::new(records),
            ranges: Arc::new(ranges),
            visited,
        }
    }
}
#[cfg(all(test, feature = "tree-sitter-rust"))]
mod tests {
    use super::*;
    fn point(s: &str, p: usize) -> tree_sitter::Point {
        let s = &s[..p];
        tree_sitter::Point::new(
            s.bytes().filter(|b| *b == b'\n').count(),
            s.rsplit('\n').next().unwrap().len(),
        )
    }
    fn edit(a: &str, b: &str) -> InputEdit {
        let mut start = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
        while !a.is_char_boundary(start) || !b.is_char_boundary(start) {
            start -= 1;
        }
        let mut suffix = a[start..]
            .bytes()
            .rev()
            .zip(b[start..].bytes().rev())
            .take_while(|(x, y)| x == y)
            .count();
        while !a.is_char_boundary(a.len() - suffix) || !b.is_char_boundary(b.len() - suffix) {
            suffix -= 1;
        }
        InputEdit {
            start_byte: start,
            old_end_byte: a.len() - suffix,
            new_end_byte: b.len() - suffix,
            start_position: point(a, start),
            old_end_position: point(a, a.len() - suffix),
            new_end_position: point(b, b.len() - suffix),
        }
    }
    #[test]
    fn folds_equal_full_after_insert_delete_paste_undo_and_block_changes() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let mut source = "// hi\nfn a() {\n let x = 1;\n}\nfn b() {\n let y = 2;\n}\n".to_string();
        let mut tree = parser.parse(&source, None).unwrap();
        let mut cache = Cache::update(None, None, &tree, &Rope::from(source.as_str()), &[], 0);
        for next in [
            "// hello\nfn a() {\n let x = 1;\n}\nfn b() {\n let y = 2;\n}\n",
            "// hello\n// newline\nfn a() {\n let x = 1;\n}\nfn b() {\n let y = 2;\n}\n",
            "// hi\nfn a() {\n if true {\n  let x = 1;\n }\n}\nfn b() {\n let y = 2;\n}\n",
            "// hi\nfn a() {\n let x = 1;\n}\nfn b() {\n let y = 2;\n}\n",
            "fn pasted() {\n loop {\n break;\n }\n}\n",
            "fn pasted() {\n loop {\n break;\n \n}\n",
            "fn pasted() {\n loop {\n break;\n }\n}\n",
            "fn one() { let x = 1; }fn two() {\n let y=2;\n}\n",
            "fn one() {\n let x = 1;\n}fn two() {\n let y=2;\n}\n",
            "fn one() { let x = 1; }fn two() {\n let y=2;\n}\n",
            "",
        ] {
            let e = edit(&source, next);
            let mut old = tree.clone();
            old.edit(&e);
            tree = parser.parse(next, Some(&old)).unwrap();
            let result = Cache::update(
                Some(&cache),
                Some(&old),
                &tree,
                &Rope::from(next),
                &[e],
                cache.revision + 1,
            );
            assert_eq!(
                *result.ranges,
                super::super::extract_fold_ranges(&tree),
                "{next}"
            );
            if next.starts_with("// hello\nfn") {
                assert_eq!(
                    result.visited, 0,
                    "nonstructural edit must not traverse the tree"
                );
            }
            cache = result;
            source = next.to_owned();
        }
    }
    #[test]
    fn unchanged_tree_and_other_functions_reuse_the_cache() {
        let source = (0..500)
            .map(|i| format!("fn f{i}() {{\n let x = 1;\n}}\n"))
            .collect::<String>();
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        let tree = p.parse(&source, None).unwrap();
        let cache = Cache::update(None, None, &tree, &Rope::from(source.as_str()), &[], 0);
        let same = Cache::update(
            Some(&cache),
            Some(&tree),
            &tree,
            &Rope::from(source.as_str()),
            &[],
            0,
        );
        assert_eq!(same.visited, 0);
        assert!(Arc::ptr_eq(&same.ranges, &cache.ranges));
        let renamed = source.replacen("let x", "let longer_identifier", 1);
        let edit_name = edit(&source, &renamed);
        let mut edited = tree.clone();
        edited.edit(&edit_name);
        let renamed_tree = p.parse(&renamed, Some(&edited)).unwrap();
        assert!(edited.changed_ranges(&renamed_tree).next().is_none());
        let rename_cache = Cache::update(
            Some(&cache),
            Some(&edited),
            &renamed_tree,
            &Rope::from(renamed.as_str()),
            &[edit_name],
            1,
        );
        assert_eq!(
            *rename_cache.ranges,
            super::super::extract_fold_ranges(&renamed_tree)
        );
        assert!(
            Arc::ptr_eq(&rename_cache.ranges, &cache.ranges),
            "unchanged line geometry must reuse fold ranges"
        );
        assert_eq!(rename_cache.visited, 0);
        let next = source.replacen(" let x = 1;", " if true {\n let x = 1;\n }", 1);
        let e = edit(&source, &next);
        let mut old = tree.clone();
        old.edit(&e);
        let new = p.parse(&next, Some(&old)).unwrap();
        let result = Cache::update(
            Some(&cache),
            Some(&old),
            &new,
            &Rope::from(next.as_str()),
            &[e],
            1,
        );
        assert_eq!(*result.ranges, super::super::extract_fold_ranges(&new));
        assert!(
            result.visited < cache.visited / 10,
            "only the affected function should be traversed"
        );
    }
}
