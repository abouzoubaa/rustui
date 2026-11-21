// PX-3: conservative negative-result reuse for the pinned Rust injection query.
use super::*;
use std::sync::OnceLock;

// Deliberately literal: a changed repository query must not silently opt in.
const PINNED_QUERY: &str = "((macro_invocation\n  (token_tree) @injection.content)\n (#set! injection.language \"rust\")\n (#set! injection.include-children))\n\n((macro_rule\n  (token_tree) @injection.content)\n (#set! injection.language \"rust\")\n (#set! injection.include-children))";

pub(super) struct State {
    pub(super) enabled: bool,
    eligible: bool,
    generation: u64,
    query: Option<Arc<Query>>,
    empty: bool,
    #[cfg(test)]
    pub(super) hits: usize,
}

impl State {
    pub(super) fn new(
        config: &super::super::LanguageConfig,
        grammar: &tree_sitter::Language,
        query: Option<&Arc<Query>>,
        generation: u64,
    ) -> Self {
        static ON: OnceLock<bool> = OnceLock::new();
        let enabled = *ON.get_or_init(|| std::env::var("RUSTUI_HL_FASTPATH").as_deref() == Ok("1"));
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
            #[cfg(test)]
            hits: 0,
        }
    }

    pub(super) fn inert() -> Self {
        Self {
            enabled: false,
            eligible: false,
            generation: 0,
            query: None,
            empty: false,
            #[cfg(test)]
            hits: 0,
        }
    }

    pub(super) fn can_reuse(
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
            && self.empty
            && same_grammar
            && self.generation == LanguageRegistry::singleton().generation()
            && self
                .query
                .as_ref()
                .zip(query)
                .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
    }

    pub(super) fn observed(&mut self, zero_matches: bool) {
        self.empty = zero_matches;
    }
    pub(super) fn invalidate(&mut self) {
        self.empty = false;
    }
    pub(super) fn reused(&mut self) {
        #[cfg(test)]
        {
            self.hits += 1;
        }
    }
}
