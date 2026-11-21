use super::*;

fn point(source: &str, offset: usize) -> Point {
    let prefix = &source[..offset];
    Point::new(
        prefix.bytes().filter(|b| *b == b'\n').count(),
        prefix.rsplit('\n').next().unwrap().len(),
    )
}

fn changed(highlighter: &mut SyntaxHighlighter, next: &str) {
    let old = highlighter.text().to_string();
    let start = old
        .bytes()
        .zip(next.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = old[start..]
        .bytes()
        .rev()
        .zip(next[start..].bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let edit = InputEdit {
        start_byte: start,
        old_end_byte: old.len() - suffix,
        new_end_byte: next.len() - suffix,
        start_position: point(&old, start),
        old_end_position: point(&old, old.len() - suffix),
        new_end_position: point(next, next.len() - suffix),
    };
    assert!(highlighter.update(Some(edit), &Rope::from(next), None));
}

fn captures(h: &SyntaxHighlighter) -> Vec<(Range<usize>, SharedString)> {
    h.match_styles(0..h.text.len())
        .into_iter()
        .map(|item| (item.range, item.name))
        .collect()
}

fn oracle(h: &SyntaxHighlighter, source: &str) {
    let mut full = SyntaxHighlighter::new(h.language());
    full.rust_empty.enabled = false;
    // Preserve an explicitly replaced query in the query-change test.
    full.injections_query = h.injections_query.clone();
    assert!(full.update(None, &Rope::from(source), None));
    assert_eq!(captures(h), captures(&full));
    let theme = HighlightTheme::default_dark();
    assert_eq!(
        h.styles(&(0..source.len()), theme.as_ref()),
        full.styles(&(0..source.len()), theme.as_ref())
    );
    let layers = |x: &SyntaxHighlighter| {
        x.injection_layers
            .iter()
            .map(|l| {
                (
                    l.language_name.clone(),
                    l.byte_range.clone(),
                    l.ranges.clone(),
                    l.tree.root_node().to_sexp(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(layers(h), layers(&full));
}

fn rust(source: &str) -> SyntaxHighlighter {
    let mut h = SyntaxHighlighter::new("rust");
    h.rust_empty.enabled = true;
    assert!(h.update(None, &Rope::from(source), None));
    oracle(&h, source);
    h
}

#[test]
fn px3_comment_identifier_reuse_and_undo_oracle() {
    let start = "// hello\nfn main() { let value = 1; }\n";
    let mut h = rust(start);
    for (i, text) in [
        "// hello there\nfn main() { let value = 1; }\n",
        "// hello there\nfn main() { let renamed = 1; }\n",
        start,
    ]
    .into_iter()
    .enumerate()
    {
        changed(&mut h, text);
        oracle(&h, text);
        assert_eq!(
            h.rust_empty.hits,
            [1, 2, 2][i],
            "comment/identifier reuse must run; a multi-node undo may scan"
        );
    }
}

#[test]
fn px3_macro_creation_removal_nested_tokens_and_delimiters_oracle() {
    let plain = "// hello\nfn main() { let value = 1; }\n";
    let mut h = rust(plain);
    let mut previous_hits = h.rust_empty.hits;
    for text in [
        "// hello\nfn main() { println!(\"hi\"); }\n",
        "// hello\nfn main() { println!((value + nested!(1))); }\n",
        "// hello\nfn main() { println!([value, nested!({2})]); }\n",
        "// hello\nfn main() { println!([value, nested!({2})]);\n }\n",
        "// hello\nfn main() { println!([value, nested!({2})]);\n ",
        plain,
    ] {
        changed(&mut h, text);
        oracle(&h, text);
        assert_eq!(
            h.rust_empty.hits, previous_hits,
            "macro/error/structural edits must scan"
        );
        previous_hits = h.rust_empty.hits;
    }
    changed(&mut h, "// hello again\nfn main() { let value = 1; }\n");
    assert_eq!(
        h.rust_empty.hits,
        previous_hits + 1,
        "removal must establish fresh negative evidence"
    );
}

#[test]
fn px3_macro_rule_and_newline_undo_oracle() {
    let mut h = rust("macro_rules! m { ($x:expr) => { $x }; }\nfn main() {}\n");
    for text in [
        "macro_rules! m { ($x:expr) => { ($x + 1) }; }\nfn main() {}\n",
        "macro_rules! m { ($x:expr) => { ($x + 1) }; }\n\nfn main() {}\n",
        "macro_rules! m { ($x:expr) => { $x }; }\nfn main() {}\n",
    ] {
        changed(&mut h, text);
        oracle(&h, text);
        assert_eq!(h.rust_empty.hits, 0);
    }
}

#[test]
fn px3_query_change_and_filtered_matches_are_not_negative_evidence() {
    let mut h = rust("// hello\nfn main() { let value = 1; }\n");
    h.injections_query = Some(Arc::new(
        Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            "(identifier) @injection.content",
        )
        .unwrap(),
    ));
    changed(&mut h, "// hello again\nfn main() { let value = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
    assert!(
        h.injection_layers.is_empty(),
        "unresolved language is filtered"
    );
    let data = h.injection_parse_data().unwrap();
    let (layers, zero) =
        SyntaxHighlighter::compute_injection_layers_observed(data, h.tree().unwrap(), h.text());
    assert!(layers.is_empty());
    assert!(!zero, "actual matches before filtering must be counted");
    changed(&mut h, "// hello once more\nfn main() { let value = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
}

#[test]
fn px3_changed_grammar_cannot_carry_rust_negative_result() {
    let h = rust("// hello\nfn main() {}\n");
    let mut json = SyntaxHighlighter::new("json");
    json.rust_empty = h.rust_empty; // deliberately stale cache from the previous grammar
    assert!(json.update(None, &Rope::from("{\"value\": 1}"), None));
    changed(&mut json, "{\"renamed\": 1}");
    oracle(&json, "{\"renamed\": 1}");
    assert_eq!(json.rust_empty.hits, 0);
}

#[test]
fn px3_unknown_batch_stale_tree_background_and_registry_fallbacks() {
    let start = "// hello\nfn main() { let value = 1; }\n";
    let mut h = rust(start);
    let next = "// different\nfn main() { let value = 1; }\n";
    assert!(h.update(None, &Rope::from(next), None));
    oracle(&h, next);
    assert_eq!(h.rust_empty.hits, 0);
    h.edit_tree(
        None,
        &Rope::from("// pending\nfn main() { let value = 1; }\n"),
    );
    changed(&mut h, "// pending longer\nfn main() { let value = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
    // Background output has no zero-query-match evidence, even if its layers are empty.
    let tree = h.tree().unwrap().clone();
    let text = h.text().clone();
    h.apply_background_tree(tree, &text, Vec::new());
    changed(&mut h, "// pending longest\nfn main() { let value = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
    let config = LanguageRegistry::singleton().language("rust").unwrap();
    LanguageRegistry::singleton().register("px3-generation-test", &config);
    changed(&mut h, "// pending now\nfn main() { let value = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
}

#[test]
fn px3_default_off_does_not_reuse() {
    let mut h = rust("// hello\nfn main() {}\n");
    h.rust_empty.enabled = false;
    changed(&mut h, "// hello there\nfn main() {}\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 0);
}

#[test]
fn px3_multiple_edits_scan_then_reestablish_negative_result() {
    let mut h = rust("// hello\nfn main() { let value = 1; }\n");
    let middle = "// hello there\nfn main() { let value = 1; }\n";
    let next = "// hello there\nfn main() { let renamed = 1; }\n";
    let start = middle.find("value").unwrap();
    let edits = [
        InputEdit {
            start_byte: 8,
            old_end_byte: 8,
            new_end_byte: 14,
            start_position: Point::new(0, 8),
            old_end_position: Point::new(0, 8),
            new_end_position: Point::new(0, 14),
        },
        InputEdit {
            start_byte: start,
            old_end_byte: start + 5,
            new_end_byte: start + 7,
            start_position: point(middle, start),
            old_end_position: point(middle, start + 5),
            new_end_position: point(next, start + 7),
        },
    ];
    assert!(h.update_edits(&edits, &Rope::from(next), None));
    oracle(&h, next);
    assert_eq!(h.rust_empty.hits, 0);
    changed(&mut h, "// hello again\nfn main() { let renamed = 1; }\n");
    oracle(&h, &h.text.to_string());
    assert_eq!(h.rust_empty.hits, 1);
}
