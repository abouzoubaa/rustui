//! Conditional stack grammar with explicit keyed native editor identity.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("conditional-gallery", ConditionalGallery::default)
}

#[component]
struct ConditionalGallery {
    #[state(0)]
    mode: usize,
    #[state(false)]
    reversed: bool,
    #[state(true)]
    show_optional: bool,
    #[state(0)]
    parent_updates: usize,
    #[state(0)]
    owner_hits: usize,
    #[state(String::from("Editor A"))]
    first: String,
    #[state(String::from("Editor B"))]
    second: String,
    #[state(String::from("Optional editor"))]
    optional: String,
    #[state(String::from("Stable tail"))]
    tail: String,
    #[state(String::from("Fallback branch"))]
    fallback: String,
}

impl ConditionalGallery {
    fn cycle_mode(&mut self) {
        self.mode = (self.mode + 1) % 3;
    }

    fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }

    fn toggle_optional(&mut self) {
        self.show_optional = !self.show_optional;
    }

    fn update_parent(&mut self) {
        self.parent_updates += 1;
    }

    fn hit(&mut self) {
        self.owner_hits += 1;
    }
}

impl View for ConditionalGallery {
    fn body(&self) -> impl View {
        VStack! {
            Text("Conditional stacks: if / else-if / else")
            super::locale_selector::LocaleSelector::new()
            HStack! {
                Button("Cycle branch", cycle_mode).id("conditional-mode")
                Button("Reverse editors", reverse).id("conditional-reverse")
                Button("Omit / restore optional", toggle_optional).id("conditional-optional")
                Button("Update parent", update_parent).id("conditional-parent")
            }.spacing(6.0)
            Text(format!(
                "Branch {} · owner hits {} · parent updates {}",
                self.mode(), self.owner_hits(), self.parent_updates()
            ))
            if self.mode() == 0 {
                Text("First branch contains multiple sibling views.")
                Button("First branch action", hit).id("conditional-first-action")
            } else if self.mode() == 1 {
                HStack! {
                    Text("Second branch nests a horizontal stack.")
                    if self.show_optional() {
                        Button("Nested conditional action", hit).id("conditional-nested-action")
                    } else {
                        Text("Nested action omitted")
                    }
                }.spacing(6.0)
            } else {
                Keyed("fallback-branch", TextField("Fallback branch", self.fallback_binding())
                    .id("conditional-fallback"))
            }
            if self.show_optional() {
                Keyed("optional-editor", TextEditor("Optional editor", self.optional_binding())
                    .id("conditional-shared-editor").height(72.0))
            }
            Text("Editors A and B keep their explicit keys while their order changes:")
            if self.reversed() {
                Keyed("editor-b", TextEditor("Editor B", self.second_binding())
                    .id("conditional-shared-editor").height(72.0))
                Keyed("editor-a", TextEditor("Editor A", self.first_binding())
                    .id("conditional-shared-editor").height(72.0))
            } else {
                Keyed("editor-a", TextEditor("Editor A", self.first_binding())
                    .id("conditional-shared-editor").height(72.0))
                Keyed("editor-b", TextEditor("Editor B", self.second_binding())
                    .id("conditional-shared-editor").height(72.0))
            }
            Keyed("stable-tail", TextField("Stable tail", self.tail_binding())
                .id("conditional-shared-editor"))
            Text(format!("Mirrors: A [{}] · B [{}] · optional [{}] · tail [{}]",
                self.first(), self.second(), self.optional(), self.tail()))
            Text("Edit different values, reverse A/B, cycle branches and update the parent. Explicit keys preserve A/B and the tail. Omitting the optional editor releases its native editing state; its owner-bound text remains for restoration. Branches add no identity scope or layout box.")
        }.spacing(8.0)
    }
}
