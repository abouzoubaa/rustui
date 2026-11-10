//! Single-stroke shortcuts scoped to the focused descendant of a RustUI view.

use std::{error::Error, fmt};

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, KeyDownEvent, KeybindingKeystroke,
    Keystroke, ParentElement as _, div,
};
use rustui_core::Identity;

use crate::view::{
    ControlAction, EmptyView, KeyboardActionHandler, RenderContext, RetentionScope, View,
};

#[derive(Clone)]
struct ShortcutBinding {
    chord: KeybindingKeystroke,
    action: ControlAction,
    enabled: bool,
}

/// A wrapper whose shortcuts run only while focus is in its child subtree.
///
/// A native control's handled key binding takes precedence. The wrapper does
/// not acquire focus, register global bindings, or add a tab stop.
pub struct KeyboardShortcutsView<V: View> {
    id: Identity,
    child: V,
    bindings: Vec<ShortcutBinding>,
}

impl<V: View> KeyboardShortcutsView<V> {
    pub fn new(id: impl Into<Identity>, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "KeyboardShortcuts identity must be nonempty"
        );
        Self {
            id,
            child,
            bindings: Vec::new(),
        }
    }

    /// Adds a single command/control chord. Invalid or duplicate chords panic.
    pub fn shortcut(self, chord: &str, action: impl Into<ControlAction>) -> Self {
        self.try_shortcut_enabled(chord, action, true)
            .unwrap_or_else(|error| panic!("invalid keyboard shortcut: {error}"))
    }

    /// Adds a chord that can be disabled without changing this wrapper's ID.
    pub fn shortcut_enabled(
        self,
        chord: &str,
        action: impl Into<ControlAction>,
        enabled: bool,
    ) -> Self {
        self.try_shortcut_enabled(chord, action, enabled)
            .unwrap_or_else(|error| panic!("invalid keyboard shortcut: {error}"))
    }

    /// Validates and adds a chord without panicking.
    pub fn try_shortcut_enabled(
        mut self,
        chord: &str,
        action: impl Into<ControlAction>,
        enabled: bool,
    ) -> Result<Self, KeyboardShortcutError> {
        let chord = parse_chord(chord)?;
        if self.bindings.iter().any(|binding| binding.chord == chord) {
            return Err(KeyboardShortcutError::DuplicateChord);
        }
        let action = action.into();
        if matches!(&action, ControlAction::Application(action) if !action.keyboard_compatible()) {
            return Err(KeyboardShortcutError::ClickOnlyAction);
        }
        self.bindings.push(ShortcutBinding {
            chord,
            action,
            enabled,
        });
        Ok(self)
    }
}

#[allow(non_snake_case)]
pub fn KeyboardShortcuts<V: View>(id: impl Into<Identity>, child: V) -> KeyboardShortcutsView<V> {
    KeyboardShortcutsView::new(id, child)
}

impl<V: View> View for KeyboardShortcutsView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Keyboard(self.id.clone()), |context| {
            let bindings: Vec<_> = self
                .bindings
                .iter()
                .filter(|binding| binding.enabled)
                .map(|binding| {
                    (
                        binding.chord.clone(),
                        context.bind_keyboard_action(binding.action.clone()),
                    )
                })
                .collect();
            let child = self.child.into_element(context);
            div()
                .id(format!("rustui-keyboard-shortcuts-{}", self.id.as_str()))
                .on_key_down(move |event, window, cx| {
                    if let Some(action) = matching_action(event, &bindings) {
                        action(window, cx);
                        cx.stop_propagation();
                    }
                })
                .child(child)
                .into_any_element()
        })
    }
}

type BoundShortcut = (KeybindingKeystroke, KeyboardActionHandler);

fn matching_action<'a>(
    event: &KeyDownEvent,
    bindings: &'a [BoundShortcut],
) -> Option<&'a KeyboardActionHandler> {
    if event.is_held || event.prefer_character_input || event.keystroke.is_ime_in_progress() {
        return None;
    }
    bindings
        .iter()
        .find(|(chord, _)| event.keystroke.should_match(chord))
        .map(|(_, action)| action)
}

fn parse_chord(chord: &str) -> Result<KeybindingKeystroke, KeyboardShortcutError> {
    if chord.trim() != chord || chord.chars().any(char::is_whitespace) || chord.is_empty() {
        return Err(KeyboardShortcutError::InvalidChord);
    }
    let keystroke = Keystroke::parse(chord).map_err(|_| KeyboardShortcutError::InvalidChord)?;
    if keystroke.key_char.is_some() {
        return Err(KeyboardShortcutError::InvalidChord);
    }
    if !(keystroke.modifiers.control || keystroke.modifiers.platform) {
        return Err(KeyboardShortcutError::RequiresCommandOrControl);
    }
    Ok(KeybindingKeystroke::from_keystroke(keystroke))
}

/// A shortcut definition that cannot be safely installed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardShortcutError {
    InvalidChord,
    RequiresCommandOrControl,
    DuplicateChord,
    ClickOnlyAction,
}

impl fmt::Display for KeyboardShortcutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidChord => "expected one GPUI keystroke such as secondary-shift-k",
            Self::RequiresCommandOrControl => "shortcut requires command or control",
            Self::DuplicateChord => "duplicate chord in the same keyboard scope",
            Self::ClickOnlyAction => "action requires a click event",
        })
    }
}

impl Error for KeyboardShortcutError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::AppAction;

    fn event(chord: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke::parse(chord).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[test]
    fn validates_native_single_chords_and_rejects_duplicate_spellings() {
        assert!(matches!(
            parse_chord("k"),
            Err(KeyboardShortcutError::RequiresCommandOrControl)
        ));
        assert!(matches!(
            parse_chord("ctrl-k ctrl-c"),
            Err(KeyboardShortcutError::InvalidChord)
        ));
        assert!(matches!(
            parse_chord("cmd-k->x"),
            Err(KeyboardShortcutError::InvalidChord)
        ));
        let scope = KeyboardShortcuts("scope", EmptyView)
            .try_shortcut_enabled("secondary-shift-k", AppAction::new(|_| {}), true)
            .unwrap();
        assert!(matches!(
            scope.try_shortcut_enabled("secondary-shift-K", AppAction::new(|_| {}), false),
            Err(KeyboardShortcutError::DuplicateChord)
        ));
    }

    #[test]
    fn matches_only_unheld_noncharacter_input() {
        let chord = parse_chord("secondary-shift-k").unwrap();
        let bindings: Vec<BoundShortcut> = vec![(chord, Box::new(|_, _| {}))];
        let matched = event("secondary-shift-k");
        assert!(matching_action(&matched, &bindings).is_some());
        assert!(matching_action(&event("secondary-shift-j"), &bindings).is_none());
        assert!(matching_action(&event("alt-shift-k"), &bindings).is_none());
        let mut typing = event("k");
        typing.keystroke.key_char = Some("k".into());
        assert!(matching_action(&typing, &bindings).is_none());
        assert!(matching_action(&event("k"), &bindings).is_none());
        assert!(
            matching_action(
                &KeyDownEvent {
                    is_held: true,
                    ..matched.clone()
                },
                &bindings
            )
            .is_none()
        );
        assert!(
            matching_action(
                &KeyDownEvent {
                    prefer_character_input: true,
                    ..matched
                },
                &bindings
            )
            .is_none()
        );
        let mut composing = event("secondary-shift-k");
        composing.keystroke.key_char = Some("k".into());
        composing.prefer_character_input = true;
        assert!(matching_action(&composing, &bindings).is_none());
    }

    #[test]
    fn rejects_click_only_application_actions_even_when_composed() {
        let click_only = AppAction::with_window_action(|_, _, _| {});
        let action = AppAction::new(|_| {}).then(click_only);
        assert!(matches!(
            KeyboardShortcuts("scope", EmptyView).try_shortcut_enabled(
                "secondary-shift-k",
                action,
                true
            ),
            Err(KeyboardShortcutError::ClickOnlyAction)
        ));
        assert!(
            AppAction::new(|_| {})
                .then(AppAction::new(|_| {}))
                .keyboard_compatible()
        );
    }
}
