// PX-3: conservative negative-result reuse for the pinned Rust injection query.
use super::*;
use std::sync::OnceLock;

// Deliberately literal: a changed repository query must not silently opt in.
const PINNED_QUERY: &str = "((macro_invocation\n  (token_tree) @injection.content)\n (#set! injection.language \"rust\")\n (#set! injection.include-children))\n\n((macro_rule\n  (token_tree) @injection.content)\n (#set! injection.language \"rust\")\n (#set! injection.include-children))";

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) enabled: bool,
    eligible: bool,
    generation: u64,
    query: Option<Arc<Query>>,
    empty: bool,
    pub(crate) captures: Option<Arc<[super::rust_injection_ranges::Capture]>>,
    pub(crate) trace: Option<super::rust_injection_ranges::Trace>,
    #[cfg(test)]
    pub(crate) hits: usize,
}

impl State {
    pub(crate) fn new(
        config: &super::super::LanguageConfig,
        grammar: &tree_sitter::Language,
        query: Option<&Arc<Query>>,
        generation: u64,
    ) -> Self {
        static ON: OnceLock<bool> = OnceLock::new();
        let enabled = *ON.get_or_init(|| {
            gpui::fast_g5_profile_enabled()
                || std::env::var("RUSTUI_HL_FASTPATH").as_deref() == Ok("1")
        });
        #[cfg(feature = "tree-sitter-rust")]
        let eligible = config.name.as_ref() == "rust"
            && config.injections.as_ref() == PINNED_QUERY
            && grammar == &tree_sitter_rust::LANGUAGE.into();
        #[cfg(not(feature = "tree-sitter-rust"))]
        let eligible = {
            let _ = (config, grammar, PINNED_QUERY);
            false
        };
        Self {
            enabled,
            eligible,
            generation,
            query: query.cloned(),
            empty: false,
            captures: None,
            trace: None,
            #[cfg(test)]
            hits: 0,
        }
    }

    pub(crate) fn inert() -> Self {
        Self {
            enabled: false,
            eligible: false,
            generation: 0,
            query: None,
            empty: false,
            captures: None,
            trace: None,
            #[cfg(test)]
            hits: 0,
        }
    }

    pub(crate) fn can_reuse(
        &self,
        query: Option<&Arc<Query>>,
        grammar: Option<&tree_sitter::Language>,
    ) -> bool {
        self.empty && self.current(query, grammar)
    }

    pub(crate) fn current(
        &self,
        query: Option<&Arc<Query>>,
        grammar: Option<&tree_sitter::Language>,
    ) -> bool {
        #[cfg(feature = "tree-sitter-rust")]
        let same_grammar = grammar.is_some_and(|g| g == &tree_sitter_rust::LANGUAGE.into());
        #[cfg(not(feature = "tree-sitter-rust"))]
        let same_grammar = {
            let _ = grammar;
            false
        };
        self.enabled
            && self.eligible
            && same_grammar
            && self.generation == LanguageRegistry::singleton().generation()
            && self
                .query
                .as_ref()
                .zip(query)
                .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
    }

    /// Diagnose every legacy predicate independently, including ones that
    /// would be short-circuited by an earlier failure.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn failed_predicates(
        &self,
        query: &Arc<Query>,
        grammar: &tree_sitter::Language,
        edit_count: usize,
        old: Option<&Tree>,
        new: &Tree,
        structural_changes: bool,
    ) -> u16 {
        #[cfg(feature = "tree-sitter-rust")]
        let same_grammar = grammar == &tree_sitter_rust::LANGUAGE.into();
        #[cfg(not(feature = "tree-sitter-rust"))]
        let same_grammar = {
            let _ = grammar;
            false
        };
        [
            edit_count != 1,
            !self.enabled,
            !self.eligible,
            !self.empty,
            !same_grammar,
            self.generation != LanguageRegistry::singleton().generation(),
            !self
                .query
                .as_ref()
                .is_some_and(|saved| Arc::ptr_eq(saved, query)),
            new.root_node().has_error(),
            old.is_none(),
            old.is_some_and(|old| old.root_node().has_error()),
            structural_changes,
        ]
        .into_iter()
        .enumerate()
        .fold(0, |flags, (bit, failed)| flags | (u16::from(failed) << bit))
    }

    pub(crate) fn observed(&mut self, zero_matches: bool) {
        self.empty = zero_matches;
    }
    pub(crate) fn invalidate(&mut self) {
        self.empty = false;
        self.captures = None;
        self.trace = None;
    }
    pub(crate) fn reused(&mut self) {
        #[cfg(test)]
        {
            self.hits += 1;
        }
    }
}
