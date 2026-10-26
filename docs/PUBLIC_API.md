# RustUI public API

This document records the implemented public surface and the next contracts. It
does not treat scaffolded module paths as supported APIs.

## Implemented surface

The current prelude exports, among other names:

```rust
use rustui::prelude::{
    component, AppAction, Binding, Button, ButtonStyle, Component, ComponentView, HStack, Identity,
    AnimatedOpacity, Collection, CollectionItem, KeyedList, KeyedListItem, KeyedListView,
    LayoutDirection, Locale,
    LocalizedText, Menu, MenuItem, MenuView, OpacityAnimationError, Popover, PopoverView, RustUI,
    Sidebar, SidebarItem, SidebarView, NavigationSplit, NavigationSplitItem,
    Offset, OffsetView,
    NavigationSplitView, State, Text, Tooltip, TooltipView, VStack, View, ViewHost,
    Alert, AlertVariant, AlertView, FontFallbackPolicy, FontFallbackPolicyError, Toast,
    ToastBuilder, ToastKind, Toolbar, ToolbarItem, ToolbarView, CodeEditor, CodeEditorView,
    FocusScope, FocusScopeView, KeyboardShortcutError, KeyboardShortcuts,
    KeyboardShortcutsView, Frame, FrameView, LazyGrid, LazyGridItem,
    SpringConfig, SpringOpacity, SpringOpacityError, SpringOffset, SpringOffsetError, GeometryReader, GeometrySize,
    HoverActionError, HoverRegion, HoverRegionView,
    DragOperation, DragSource, DragSourceView, DropEvent, DropTarget, DropTargetView,
    Shadow, ShadowElevation, ShadowStyle, ShadowStyleError, ShadowView,
    Card, CardView,
};
```

Implemented developer syntax:

```rust
#[component]
struct Counter {
    #[state(0)]
    count: i32,
}

impl Counter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for Counter {
    fn body(&self) -> impl View {
        VStack! {
            Text(format!("Count: {}", self.count()))
            Button("Increment", increment)
                .button_style(ButtonStyle::BorderedProminent)
        }
    }
}
```

Inside `VStack!` and `HStack!`, the bare Button handler is expanded to the
owning component action. Direct construction outside that DSL currently requires
the internal action form and is not yet the final ergonomic contract.

`Text`, `Button`, `VStack`, and `HStack` lower into GPUI elements. `Text`
supports explicit font size, RustUI font weights, and
`.font_fallbacks(policy)` on both `Text(...)` and `LocalizedText(...)`, where
`policy` is a validated `FontFallbackPolicy::new([...])` result.
The policy keeps the caller's ordered family names and rejects an empty list or
blank family names; GPUI owns actual font resolution and shaping. It does not
select families by locale or establish OS font-selection provenance. In the
bounded M42 native sample, Latin `Hello`, connected Arabic `مرحبًا`, Chinese
`你好`, and French `LocalizedText` `Fermer` rendered with the policy set. This
does not identify which OS fonts supplied the glyphs; native fallback samples
in other locales remain pending. See
`target/headstart/milestone42-text-fallback-gui.json`. `Button` supports stable identity,
style, disabled state, literal or localized accessibility labels, hover help, tab order,
and focus traversal. `.localized_accessibility_label("button.save")` resolves through
the active message catalog and falls back to an explicit name or visible button label.
Stacks accept finite, nonnegative spacing and panic for negative or
nonfinite values. The total span across child gaps must also remain finite;
they also support cross-axis alignment. GPUI Components retains
Button pointer, Enter/Space keyboard activation, focus-ring, and accessibility behavior.
`ViewHost` retains a root component in a GPUI entity and dispatches component
mutation methods through GPUI listeners.

`Button("Save", action).localized_label("button.save")` resolves its visible
label from the active catalog at render time. The constructor label remains the
fallback and stable identity seed. Without a separate accessibility override,
the announced name follows the resolved visible label.

## Avatars and badges

`Avatar` and `Badge` support explicit or localized accessible names through
`.accessibility_label(...)` and `.localized_accessibility_label(...)`.
Localization keys must be nonblank. If a translation is blank or unresolved,
the accessible name falls back to the visible Avatar name or Badge announcement;
an explicit literal name is retained as the fallback when supplied.

## Links

`Link` supports literal or localized accessible names. Use
`.localized_accessibility_label("rustui-link-open-fixture")` to resolve a name
from the active message catalog; a missing, blank, or key-echo result falls
back to the explicit name or visible link label.

## Tooltips

`Tooltip` attaches visual descriptive text to a child view. The wrapper requires
an explicit stable ID, and literal text or localization keys must be nonblank.
Use `.localized(...)` to resolve the tooltip string from
the active message catalogs:

```rust
Tooltip("Increment the counter.", Button("Increment", self.action(Self::increment)))
    .id("increment-help")

Tooltip("unused literal", Text("Details"))
    .localized("settings-details-help")
    .id("settings-details-help")
```

Tooltip text is visual help; the adapter does not set an accessible name or
description. The Playground gallery is reachable and native layout plus the
child Button action (count 0 to 1) were observed. Tooltip display and dismissal
remain unverified: the available pointer automation did not provide pointer
movement with a hover dwell, so those attempts do not establish a failure.

## Action menus

`Menu` builds an action menu from `MenuItem::action` entries and optional
separators. Give the trigger an explicit stable ID; action IDs must be nonempty
and unique within that menu. Labels may use literals or active localization
message keys, and action items may be disabled:

```rust
Menu(
    "Actions",
    [
        MenuItem::action("increment", "Increment", self.action(Self::increment)),
        MenuItem::separator(),
        MenuItem::action("reset", "Reset", self.action(Self::reset)),
        MenuItem::action("unavailable", "Unavailable", self.action(Self::unavailable))
            .disabled(true),
    ],
)
.id("actions-menu")
```

The adapter uses the upstream GPUI Components popup for menu lifecycle and
keyboard behavior. Bounded M30 native interaction accepted keyboard and pointer
Increment, pointer Reset, and the ordinary Button smoke path. Disabled pointer
activation and Enter suppressed the action callback (0 hits); Enter still
dismisses a disabled row. Native popup row identity remains positional because
the pinned upstream API does not accept RustUI row IDs. These checks do not
establish complete accessibility, focus, or open-popup owner-removal behavior.

`ContextMenu(id, content, items)` accepts a RustUI child and the same validated
`MenuItem` actions. The ID is stable and nonempty; right-click or long press
delegates popup behavior to GPUI Components. Bounded M43 native review observed
right-click opening, Increment, disabled callback suppression, reversed item
order, and in-menu removal of the target owner followed by a fresh restore.
The open popup disappeared when its owner was removed. Detached stale-action
invocation, long press, complete keyboard focus, and screen-reader behavior
remain unverified (`target/headstart/milestone43-context-menu-gui.json`).

## Toolbars

`Toolbar(id, items)` uses the pinned GPUI Components toolbar for ordered actions
and roving arrow-key focus. Both the toolbar ID and each action ID must be stable
and nonempty; action IDs must be unique in that toolbar. Actions use the owning
component's `ControlAction` path:

```rust
Toolbar(
    "editor-toolbar",
    [
        ToolbarItem::action("save", "Save", self.action(Self::save)),
        ToolbarItem::action("close", "Close", self.action(Self::close))
            .localized("rustui-action-close"),
    ],
)
```

An item or the whole toolbar may be disabled. A disabled toolbar also suppresses
its action callbacks. This slice supports text action buttons; groups, hosted
inputs, sizing, and icon-only controls remain outside the contract. Native
keyboard, focus, visual, and accessibility acceptance remain pending.

## Inline alerts

`Alert(id, message, visible_binding)` is an inline message, not a modal dialog.
It supports an optional title, semantic `AlertVariant`, and localized title or
message keys, which must be nonblank. Its close control writes `false` to the owning visibility binding;
the owner can also change visibility externally.

```rust
Alert("network-alert", "Connection lost", self.alert_visible_binding())
    .title("Offline")
    .variant(AlertVariant::Warning)
```

The pinned GPUI Components alert owns presentation. Bounded M41 native
observations confirmed the named Close control, dismissal, French `Fermer`
name, external Hide/Show, and green Success, amber Warning, and red Error
appearances. Default/Info appearance, variant icon pixels, keyboard focus and
activation, and full alert screen-reader announcement remain unverified. See
`target/headstart/milestone41-alert-gui.json` for the per-binary record.

## In-app toasts

`Toast(message)` builds a window-scoped GPUI Components notification. `.show()`
returns an `AppAction` to pass to a RustUI control. The descriptor supports
`.title(...)`, `.kind(ToastKind::...)`, `.id(...)` for same-window replacement,
`.autohide(...)`, `.on_click(app_action)` for the body, and
`.action(label, app_action)` for the native action button.

```rust
Button(
    "Save",
    Toast("Changes saved")
        .kind(ToastKind::Success)
        .id("save-result")
        .show(),
)
```

Toast delivery is in-app only. The last completed workspace suite passed 115
tests across 12 binaries (`target/headstart/milestone40-42-tests.log`), but the
newer action-button naming source has not completed its subsequent build/test
gate. Native acceptance remains pending for that version.

## Trigger-driven dialogs

`Dialog(id, trigger)` opens a window-scoped modal dialog. The accessible name
uses the visible title when present, then the trigger label. Supply an explicit
name with `.accessibility_label("Confirm saving changes")`, or resolve one from
the active RustUI message catalog with
`.localized_accessibility_label("dialog.save.name")`. If a localized
override is unresolved, an earlier explicit literal name remains available
as a fallback; a later literal override replaces it.

The canonical Playground dialog story demonstrates an explicit accessible
name. M119's macOS accessibility tree exposed a named `dialog` node. Focus
entry, containment, restoration, and screen-reader speech still need separate
acceptance.

## Trigger-driven sheets

`Sheet(id, trigger)` opens a side sheet using GPUI Components' default placement
and size. The trigger must have a nonblank visible label; optional literal
titles and accessible-name overrides must also be nonblank. Its accessible name uses an explicit
`.accessibility_label("Details and sheet contents")` when supplied, then the
visible title, then the trigger label. Use
`.localized_accessibility_label("sheet.details.name")` to resolve the explicit
name from the active RustUI message catalog. The canonical Playground story
demonstrates an explicit name. The semantic `dialog` node remains inside the
sheet body because the upstream Sheet host does not expose its accessibility
surface; this does not name the upstream close button.

## Popovers

`Popover` presents RustUI content from a themed Button and binds whether it is
open:

```rust
Popover(
    "help-popover",
    "Open help",
    self.help_open_binding(),
    VStack! {
        Text("Help content")
        Button("Close", self.action(Self::close_help))
    },
)
.localized_label("help-popover-trigger")
```

The ID must be nonempty and stable within its composition scope. Content is
lowered in the trigger's owner context under a popover-specific retention scope
only while open; bindings, actions, localization, theme, and environment retain
their original owner behavior. Open changes write through the owner binding only
when the value changes. The pinned Popover supplies its default anchor, outside
click, Escape, focus restoration, and presentation. This slice accepts a label
for a RustUI Button trigger, not an arbitrary trigger view or placement/style
builders. In the M36 native review, a focused TextField accepted literal
spaces without dismissing the popup; owner actions, child release and remount,
owner removal and restoration, environment and locale updates, Escape, outside
click, and intrinsic trigger width were observed. Trigger Space/Enter, complete
focus traversal, and screenreader acceptance remain pending. Evidence:
`target/headstart/milestone36-popover-gui.json`.

For each `#[state(value)]` field, `#[component]` now generates a clone getter,
`set_<field>`, `update_<field>`, `<field>_binding`, and `<field>_state`. Storage
remains on the component owned by GPUI. `action_if_changed` accepts a mutation that
returns `bool`, allowing no-op updates to skip `Context::notify()`.

## Current limitations


- State access and bindings are field-backed, but derived state and field-level
  dependency tracking are not implemented; invalidation is currently whole-host.
- Environment values reach primitive lowering through `RenderContext`; ergonomic
  component property access, async state, and typed events are not implemented.
- The stack DSL accepts constructor-call expressions and chained methods. General
  `if`/`for`, tuple, and block syntax is not supported in the macros. Conditional
  and repeated content use `Option<V>`, `Keyed`, and stack `.children(...)` builders.
- VStack start/end alignment and HStack flow follow the active locale direction.
  Unicode bidi isolation and logical padding are not implemented.
- Button accepts component methods through `Action` and application-global changes
  through `AppAction`; broader typed event contracts are not implemented.
- `rustui-i18n::interpolate` supports named text/integer placeholders with
  escaped braces. Plurals, terms, select expressions, and locale-sensitive
  date/number formatting are not implemented.
- Repeated Button labels require explicit stable IDs to avoid GPUI element ID
  collisions.
- Most public module paths contain scaffolds rather than supported contracts.

### Composing application actions

`AppAction::new` creates an application-level callback. Chain actions with
`.then(...)` to run them in order from one button activation:

```rust
let prepare = AppAction::new(|cx| record_submission(cx));
let finish = AppAction::new(|cx| clear_draft(cx));
Button("Submit", prepare.then(finish))
```

Both callbacks run synchronously in the same native click dispatch and receive
the same event context; each `AppAction::new` closure receives `&mut App`.
Composition does not wait for navigation, rendering, or other asynchronous work.

## Retained text inputs

`TextField`, `SecureField`, `SearchField`, and `TextEditor` take a generated
`Binding<String>`:

```rust
#[component]
struct Form {
    #[state(String::new())]
    name: String,
}

impl View for Form {
    fn body(&self) -> impl View {
        VStack! {
            TextField("Name", self.name_binding()).id("name")
            SecureField("Password", self.name_binding()).id("password")
            SearchField("Search", self.name_binding()).id("search")
            TextEditor("Notes", self.name_binding()).id("notes")
        }
    }
}
```

Each stable ID within its composition scope owns one GPUI `InputState` retained
by the root component host.
The default identity is derived from the control kind and placeholder; repeated
controls or dynamic/localized placeholders should always set `.id(...)`.
`SecureField` enables password semantics, masking, and a RustUI-owned,
keyboard-focusable reveal control. `SearchField` enables the upstream clear
affordance. `TextEditor` retains a
multiline `TextareaState` and exposes an explicit height; `.height(...)` requires
a finite value and clamps to at least 40 logical points. Its accessible name can
resolve through the active catalog with
`.localized_accessibility_label("editor.notes")`; the explicit name or placeholder
supplies the fallback.

`TokenField` accepts separate bindings for committed tokens and the current
draft. The Add action ignores blank drafts, trims committed values, and each
token has an accessible remove action:

```rust
TokenField("Add recipients", self.recipients_binding(), self.draft_binding())
    .id("recipients")
```

The caller owns both values and should keep the field ID stable across renders.

`Toggle`, `Checkbox`, and `Radio` take `Binding<bool>` and stable IDs.
They are controlled values: GPUI Components reports the requested value, RustUI
writes it into the owning component, and the next body pass renders that value.

`Slider` and `Stepper` use numeric bindings:

```rust
VStack! {
    Slider(self.volume_binding(), 0.0..=100.0)
        .id("volume")
        .accessibility_label("Volume")
        .step(5.0)
    Stepper("Guests", self.guests_binding())
        .id("guests")
        .range(1..=12)
        .localized_accessibility_label("stepper.guests")
}
```

`Slider` retains GPUI Components `SliderState`, synchronizes external binding
changes into its entity, and writes `SliderEvent::Change` values back to the
component. Controlled values stay in the declared finite range: NaN resolves to
the minimum, while infinities and finite out-of-range values clamp to the
nearest endpoint. Give each slider a meaningful `.accessibility_label(...)`;
the semantic slider exposes its current value, range, step, and orientation,
while the label names the value control. `Stepper` is a numeric control with
increment and decrement buttons. `.accessibility_label(...)` sets the shared
name used by those buttons; `.localized_accessibility_label(...)` resolves that
name through the active RustUI catalog and falls back to the explicit name or
visible label (define `stepper.guests` in your application catalog). The button
actions use `rustui-stepper-increment` and
`rustui-stepper-decrement` catalog messages, each with a `{label}` argument.

## String selection

`Picker` and `ComboBox` provide controlled selection over unique string options:

```rust
VStack! {
    Picker(
        "Framework",
        self.framework_binding(),
        ["Rust", "Swift", "Kotlin"],
    ).id("framework")

    ComboBox(
        "Rendering engine",
        self.engine_binding(),
        ["GPUI", "AppKit", "Skia"],
    ).id("engine").search_placeholder("Find an engine")
}
```

`Picker` uses retained GPUI Components `SelectState`; it is non-searchable by
default and supports `.searchable(true)`. `ComboBox` uses retained searchable
`SelectState` with single selection. Both synchronize changes in either the
component binding or option list without replacing the retained dropdown entity.
ComboBox confirms user selections into its string binding; its visible label is
also the default accessible name. Use
`.localized_accessibility_label("combo_box.language")` to resolve an accessible
name through the active catalog, with the visible label as fallback. Duplicate
option strings are rejected because the string is the selection identity. Native
trigger naming, search filtering and Enter confirmation were accepted in M28
(`target/headstart/milestone28-combobox-gui.json`). Disabled/external-option
replacement cases remain pending.

## Date and color selection

`DatePicker` binds an optional `chrono::NaiveDate`, re-exported by the RustUI
prelude. `ColorPicker` binds an optional GPUI `Hsla`, also re-exported:

```rust
VStack! {
    DatePicker("Choose a date", self.date_binding()).id("date")
    ColorPicker("Choose a color", self.color_binding()).id("color")
}
```

Both controls retain the upstream picker entity across body rebuilds and synchronize
external binding changes without emitting user events. `DatePicker` is clearable by default. Give its trigger an explicit accessible name with `.accessibility_label(...)`, or resolve one from the active message catalog with `.localized_accessibility_label("date_picker.start")`; the explicit name or visible label remains the fallback. `ColorPicker` uses the upstream form-style `ColorSelect`; `.compact(true)`
selects the toolbar-style swatch and label presentation.

## Virtualized string list

The `List` contract renders equal-height string rows and binds selection to the
source item index:

```rust
List(
    ["Button", "Text", "TextField", "Slider", "Picker"],
    self.selection_binding(),
)
.id("components")
.height(260.0)
.searchable(true)
.search_placeholder("Find a component")
```

`List` retains GPUI Components `ListState`, virtual scrolling, measurement, search,
focus, and keyboard navigation. The binding is `Binding<Option<usize>>`; filtering
changes visible row positions while the bound value remains the original source
index. `List` remains the string-row API. Use the typed `Collection` below for
virtualized rows containing arbitrary RustUI views.

### Stable-ID list (M34)

`KeyedList` adds stable string identity and labels while preserving the existing
source-index `List` API:

```rust
KeyedList(
    "component-list",
    [KeyedListItem::new("button", "Button"), KeyedListItem::new("list", "List")],
    self.selected_id_binding(),
)
.height(260.0)
.row_height(32.0)
.searchable(true)
.search_placeholder("Find a component")
```

Selection is `Binding<Option<String>>` and follows the selected item ID through
insertion, reorder, and filtering. Removing that item clears selection. IDs must be
nonempty and unique. Viewport and row heights reject non-finite values;
finite heights clamp to at least one logical point. The list reuses GPUI Components virtualization and native
selection; native labels were verified. The upstream outer row paths remain
position-based. Rows are strings only: arbitrary stateful RustUI row/cell builders,
full screenreader acceptance, and measured render-call bounds are not provided.

## Typed virtualized collection

`Collection<T, V, F>` accepts an `Rc<[CollectionItem<T>]>` snapshot and a row
factory `Fn(usize, &T) -> V`, where each row can be any RustUI `View`. Item IDs
must be nonempty and unique; the optional label is used as the accessible row
name. The viewport and fixed row height must be finite and positive:

```rust
use std::rc::Rc;

let items: Rc<[CollectionItem<Record>]> = records
    .iter()
    .map(|record| CollectionItem::new(record.id.clone(), record.clone()).label(record.name.clone()))
    .collect::<Vec<_>>()
    .into();

Collection::new("records", items, 320.0, 36.0, |index, record| {
    RecordRow::new(index, record)
})
.selection(self.selected_id_binding())
.label("Records")
.on_confirm(self.action(Self::confirm_selected))
```

The supplied items are the authoritative snapshot: omitting an ID removes that
row and clears a selection that no longer exists. There is no built-in search
or hidden selection store. The factory can run repeatedly, including for row
zero during native measurement, so keep mutations in callbacks. Offscreen row
owners and their local child state are released; persist durable values in the
caller-owned model and pass them back through bindings.

`.selection(binding)` is optional. It enables stable-ID selection, row-click
selection, a collection tab stop, and Up/Down/Home/End/Escape navigation.
Navigation does not wrap. `.on_confirm(action)` is also optional and requires a
keyboard-compatible action. With selection enabled, Enter confirms only when
an item is selected. Without selection, `.on_confirm` still makes the
collection a tab stop and Enter invokes the action. Row clicks avoid taking
focus from a focused editor or button inside the collection. Selection writes
compare the current owner value with the value captured for the rendered
binding; this is an equality-based stale update guard, not mutation provenance,
and it does not distinguish an A-B-A mutation sequence. Use
`.select_on_click(false)` to keep unconsumed row-child clicks from changing
selection.

The adapter uses GPUI virtualization and native row measurement. Keyboard,
focus, hit testing, lifecycle persistence, screen-reader behavior, and visual
acceptance have not received native GUI acceptance. See
`crates/rustui/src/components/data/collection.rs` and
`apps/rustui-playground/src/stories/collection_gallery.rs`.

## Relative positioning

`Offset(view)` returns an `OffsetView<V>` with signed logical-pixel `.x(x)`,
`.y(y)`, and `.translation(x, y)` setters:

```rust
HStack::new()
    .child(Offset(Button("Move", action)).translation(-12.0, 8.0))
    .child(Text("Neighbor keeps its original layout slot"))
```

The modifier uses GPUI relative positioning: the translated child keeps its
layout slot, while pointer hit testing and accessibility geometry use the
translated native bounds as provided by GPUI. Non-finite coordinates panic.
This modifier does not scale or rotate views. Native visual, interaction,
hit-testing, and accessibility acceptance remain pending. See
`crates/rustui/src/modifiers/transform.rs` and
`apps/rustui-playground/src/stories/offset_gallery.rs`.

### Spring opacity (M57)

`SpringOpacity::new(id, child, target)` returns
`Result<SpringOpacity<V>, SpringOpacityError>`. Targets and optional `.from(initial)`
values must be finite and in `0..=1`. A stable ID retains GPUI spring position and
velocity while the target changes; `.from` seeds only a newly mounted identity.
`SpringConfig::new(stiffness, damping, mass)` requires positive finite values with
representable native intermediates, and `.epsilon(tolerance)` accepts
`[f32::EPSILON, 1]`. The default config is `(170, 26, 1)` with epsilon `0.001`.
GPUI owns timing, scheduling, settling, and app-level reduced motion. Presented
opacity clamps spring overshoot; opacity does not change hit testing or accessibility
semantics. Bounded native M57 acceptance showed initial full opacity, visible target
changes, a clickable child with retained activity state, and a reduced-motion
endpoint on a later observation. Exact pixel alpha, frame timing, interruption
continuity, and scheduler quiescence were not measured; immediate reduced-motion
timing and all custom configurations were not GUI-tested. Unit validation passed
in the 167-test M56/M57 suite. The child-retention namespace fix passed native
sibling-gallery checks: four wrappers sharing one TextField ID kept separate
values through parent updates, A omission/restoration, and external replacement;
Spring B Undo/Redo affected only B. Evidence and limits are recorded in
`target/headstart/milestone57-animation-identity-gui.json`, SHA-256
`bc184cdda3d2e0f0e3d134f288a3e808153ee4febc74ac2defbe6410826f8dbd`. This does
not measure animation motion, alpha, or velocity. The separate M60 scheduling
probe passed; it measures host lowerings only. Neither result establishes an
animation trajectory or CPU/GPU performance guarantee. See
`target/headstart/milestone57-spring-gui.json`,
`target/headstart/spring-opacity-verification-63179-1792336863069682000.json`,
`docs/tasks/SPRING_OPACITY.md`, and
`apps/rustui-playground/src/stories/spring_gallery.rs`.

### GeometryReader (M58)

`GeometryReader::new(id, factory)` supplies a `GeometrySize { width, height }`
after GPUI resolves the reader's container size, then lowers the factory's RustUI
child. Width and height are logical pixels. The default reader fills its available
parent; its generated content cannot establish intrinsic size, so use a bounded
parent or explicit `.width(...)` / `.height(...)` when appropriate. Explicit
dimensions must be finite and nonnegative; zero is valid. The factory may run on
layout/prepaint even when size is unchanged, so avoid owner mutations or other
observable effects and give generated children stable IDs. This is a size-only
contract, not a position or coordinate-space observer; environment, theme, and
locale are current-render snapshots. API implementation and unit checks are
recorded. Bounded native acceptance observed readable input at 540×240, root
callbacks and text binding, persistent state across 280/540 width changes,
wide-only branch release/remount, external replacement, Arabic translation, and
reader removal/reinsertion. Only explicit widths 280 and 540 and height 240 were
GUI-tested. Intrinsic, unbounded, and zero-size behavior, exact pixels, stale
callback rejection, sibling isolation, theme/environment/task propagation,
keyboard/screen-reader behavior, and performance remain unverified. See
`docs/tasks/GEOMETRY_READER.md` and
`target/headstart/milestone58-geometry-gui.json`.

### One-shot opacity animation (M35)

Optional `.max_fps(NonZeroU16)` delegates animation update throttling to GPUI.
The default remains unrestricted. This limits the animation's own scheduling;
parent updates can still render it. It is not a presented-frame rate guarantee.
Configure before mounting; use a new identity to replay the animation.

`AnimatedOpacity::new(id, child, from, to, duration)` returns
`Result<AnimatedOpacity<V>, OpacityAnimationError>`. Endpoints must be finite values
in `0..=1`, duration must be nonzero, and the identity must be unique among sibling
animated opacity modifiers. The animation runs once on mount; use a new identity to
replay it. Changing endpoints or duration while mounted does not retarget it. It
honors GPUI's app-level `reduce_motion` setting but does not read the OS preference.
Native captures verified the full-opacity endpoint with normal and reduced motion;
intermediate frames were not captured, so animation trajectory acceptance remains
pending. This one-shot API remains separate from the state-driven `SpringOpacity`
API documented above.

## Focus-scoped keyboard shortcuts (M56)

`KeyboardShortcuts(id, child)` scopes single-stroke command/control shortcuts to
focused descendants of `child`. Chain `.shortcut(chord, action)` or
`.shortcut_enabled(chord, action, enabled)`; `try_shortcut_enabled` returns
`KeyboardShortcutError` instead of panicking for invalid definitions. Duplicate
normalized chords, invalid or whitespace-containing chords, chords without command
or control, and click-only `AppAction`s are rejected. `Action` and keyboard-compatible
`AppAction` values are supported. A native handled key binding takes precedence;
held repeats, character-input-preferred events, and incomplete IME keystrokes are
ignored. The wrapper does not add focus, a tab stop, application-wide shortcuts, or
multi-stroke sequences.

Bounded M56 native checks observed inside-scope activation, disabled-action
suppression, ordinary typing, outside-scope non-activation, and native Undo without
changing shortcut counts. Unit validation passed in the 167-test M56/M57 suite.
Dispatch conflict precedence remains untested because no competing native binding
was defined. AppAction execution, nested scopes, repeat suppression, real IME,
sibling retained-state isolation, and full platform keyboard and screen-reader
behavior remain unverified. This is a scoped single-chord API, not an application
menu keymap. See `target/headstart/milestone56-keyboard-gui.json`,
`target/headstart/milestone56-57-tests.log`, `docs/tasks/KEYBOARD_SHORTCUTS.md`, and
`apps/rustui-playground/src/stories/keyboard_gallery.rs`.

## HoverRegion (M59)

`HoverRegion(id, child).on_enter(action).on_exit(action)` observes GPUI pointer
enter/exit transitions around a RustUI subtree. Either callback may be omitted;
click-only `AppAction`s are rejected. The region adds no focus target. Removal
does not promise an exit callback, so owners should clear hover state when they
remove it. Essential hover content needs a keyboard-accessible path.

Bounded M59 native checks observed entering and exiting via clicks, text entry
changing keyboard modality and exiting hover, a child button action, child text
retention through removal/restoration, and explicit owner-state clearing when
removing while hovered. Pointer-only hover dwell, AppAction callbacks, nested
regions, stale-owner callbacks, and full platform keyboard/screen-reader coverage
remain unverified. See `target/headstart/milestone59-hover-gui.json`,
`docs/tasks/HOVER_REGION.md`, and
`apps/rustui-playground/src/stories/hover_gallery.rs`.

## SplitView (M46)


```rust
SplitView::horizontal("editor")
    .panel(SplitPanel::new("sidebar", Text("Sidebar"))
        .initial_size(180.0)
        .size_range(60.0..800.0))
    .panel(SplitPanel::new("content", Text("Content")))
```

`SplitView` is the pane container. Its draggable divider is the splitter; the
interactive area is also commonly called a gutter or resize handle. The
splitter exposes its current and allowed sizes to accessibility tools and can
be adjusted with the arrow keys.

Panel IDs must be nonempty and unique. Reordering retains child owners by ID;
removal releases them, and reinsertion creates fresh owners. Axis changes reset
dimensional preferences. Topology and range changes remap measured preferences,
clamped to current ranges; exact pixel proportions are not guaranteed. Native
panel descendant element paths remain positional. See
`docs/tasks/SPLIT_VIEW_IMPLEMENTATION.md` for sizing and acceptance limits.

## Application action factories (M49)

`AppAction::reduced_motion(bool)` and `AppAction::refresh_windows()` expose
GPUI motion preferences and explicit window refresh through RustUI actions.
Diagnostics factories enable, clear, disable, or synchronously read a snapshot:
`enable_render_diagnostics`, `clear_render_diagnostics`,
`disable_render_diagnostics`, and `read_render_diagnostics_snapshot(callback)`.
Chain with `.then(...)` to preserve execution order. These actions use the existing
GPUI app context; no additional scheduling or reactive runtime is introduced.

## Structural accessibility groups (M51)

```rust
Semantics("counter-controls", HStack! {
    Text("Count")
    Button("Increment", increment)
})
.role(SemanticsRole::Group)
.label("Counter controls")
.description("Change the current count.")
```

`SemanticsRole` supports the structural `Group`, `Region`, `Section`, and
`Navigation` roles plus live `Status` and `Alert` roles. Use
`.localized_label(key)` or `.localized_description(key)` for active catalog
lookup. A label or description without an explicit role defaults to Group.
With no metadata, the wrapper adds no accessibility node. Stable nonempty IDs
must be unique among siblings; the wrapper retains a separate child scope.
Group metadata leaves each descendant control's role and name intact.
Use `.live_region_value(value, politeness)` for a literal changing status or
`.localized_live_region_value(key, politeness)` to resolve the changing value
from the active catalog on each render. Both expose the value and its
`LivePoliteness::{Polite, Assertive}` through AccessKit and assign the `Status`
role unless a later `.role(...)` call overrides it. For example, an assertive
error value can use
`.live_region_value("Save failed", LivePoliteness::Assertive).role(SemanticsRole::Alert)`.
`.role(SemanticsRole::Alert)` supplies the Alert role; it does not itself set a
live value or politeness.

Accessibility-only hiding, focus policy, interactive roles and actions are not
part of this wrapper. Screenreader announcements (including live-region
speech), keyboard traversal, description localization, reordered native node
identity, and wider platform support remain unverified; see
`docs/tasks/SEMANTICS_GROUPS.md`.

## Virtualized string table API

The initial table contract accepts string columns and rows and binds the selected row
index:

```rust
Table(
    ["Component", "Status"],
    [
        ["Button", "Implemented"],
        ["List", "Implemented"],
    ],
    self.table_selection_binding(),
)
.id("component-table")
.height(300.0)
.striped(true)
```

`Table` retains GPUI Components `TableState` and `DataTable`, including virtual rows,
measurement, scrolling, focus, keyboard navigation, and row selection. The binding is
`Binding<Option<usize>>`. Every row must contain exactly one string cell per declared
column. Sorting and column movement are disabled until RustUI exposes their mutation
contracts; arbitrary RustUI cell builders remain a later contract.

Selection is source-index based: reordering preserves the numeric index, which can then
refer to a different row; removing rows can likewise change which row occupies the
selected index, while an index beyond the remaining rows normalizes to `None`. M76
native checks observed row selection, repeat-click preservation, and explicit clearing.

## Virtualized hierarchy

`Tree` accepts RustUI-owned nodes and binds selection to a stable node ID:

```rust
Tree(
    [TreeNode::new("components", "Components")
        .expanded(true)
        .children([
            TreeNode::new("button", "Button"),
            TreeNode::new("table", "Table"),
        ])],
    self.tree_selection_binding(),
)
.id("component-tree")
.height(300.0)
```

`Tree` retains GPUI Components `TreeState`, including virtual rows, expansion, focus,
scrolling, and keyboard navigation. Its `Binding<Option<String>>` uses caller-owned
node IDs and synchronizes pointer, keyboard, and external selection changes. Node IDs
must be unique across the complete hierarchy. `.height(...)` rejects
non-finite values and clamps finite values to at least one logical point.
Supply a stable `.id(...)` when the tree's
root topology can change: with an explicit ID, replacement, insertion, removal, and
reordering preserve the retained entity. Without `.id(...)`, the fallback identity is
derived from the root count and first root ID, so changing either can remount `TreeState`
and reset its expansion, focus, and scroll state. M75 native checks observed that unchanged
expansion declarations retain their current state and explicit expansion changes apply.
Removing and restoring a branch reset its nested child to expanded. A selected descendant
takes precedence over a collapsed ancestor declaration because GPUI automatically expands
ancestors to reveal the selection.

## Controlled tabs

`TabView` binds the selected stable tab ID and renders that tab's RustUI content:

```rust
TabView(
    [
        TabItem::new("overview", "Overview", Text("Overview content")),
        TabItem::new("details", "Details", Text("Details content")),
    ],
    self.tab_selection_binding(),
)
.id("main-tabs")
.style(TabViewStyle::Segmented)
```

Pointer, overflow, and indicator behavior reuse GPUI Components. RustUI adds a
named focus group with per-tab roving focus: Left/Right (reversed for RTL) and
Home/End move focus among enabled tabs; Enter/Space activate the focused tab.
Pointer activation focuses and selects that tab. The group label defaults to
`Tabs`; callers can set `.accessibility_label(...)`. A tab’s accessible name
defaults to its visible label and can be overridden with
`TabItem::accessibility_label(...)` or resolved from the active locale catalog
with `TabItem::localized_accessibility_label(message_id)`. The literal name or
visible label remains the fallback when that catalog entry is missing. Native
focus, screen-reader announcements, tab-to-panel association, and
cross-platform acceptance remain open. The canonical Tabs Playground story demonstrates a localized per-tab name across the shipped English, French, and Arabic catalogs.
`Binding<String>` carries the caller-owned selected ID. Tab IDs must be unique.
`TabItem` erases its child behind RustUI `AnyView`, so one tab set may contain
different concrete RustUI view types without exposing GPUI elements.
## Breadcrumb actions

`Breadcrumb` presents an ordered path. Items with actions are interactive; the final
item can omit an action to represent the current location. Each item label must be
nonblank:

```rust
Breadcrumb([
    BreadcrumbItem::new("Overview").action(go_overview),
    BreadcrumbItem::new("Components").action(go_components),
    BreadcrumbItem::new("Button"),
])
.id("component-path")
```

RustUI binds each action through the owning component host while GPUI Components owns
the separators, pointer handling, disabled state, colors, and accessibility roles.
Breadcrumb item identity currently follows its stable position in the ordered path.

## Disclosure

`Disclosure(id, label, is_open, content)` renders a controlled show/hide
region. The visible `label` must not be blank or whitespace-only; it supplies
the default accessible name. Use `.accessibility_label(...)` to set a nonblank
explicit accessible name, and `.disabled(true)` to prevent toggling while
leaving the binding unchanged.

## Navigation stack

`NavigationStack` renders its root when the bound path is empty and the destination
matching the last route when it is not:

```rust
NavigationStack(
    VStack! {
        Text("Root")
        NavigationLink("Open details", "details", self.path_binding())
    },
    [NavigationDestination::new("details", Text("Details"))],
    self.path_binding(),
)
.id("main-navigation")
```

`NavigationLink` appends its route and the stack's Back control removes the last route.
The path is `Binding<Vec<String>>`; destination route IDs must be unique. RustUI owns
the declarative path contract while GPUI owns rendering, button interaction, and host
invalidation.

### Navigation state and presentation

NavigationLink push and Back pop read the owner's current path at callback time.
They use RustUI Button, including its selected-theme metrics and accessibility
label behavior. Back uses the framework catalog's `rustui-navigation-back` label.
An unknown last route renders `rustui-navigation-unavailable-destination` and the
route text, preserves the path and keeps Back available. Rendering does not repair
or clear navigation state. With no active localization catalog, the existing
message lookup fallback still applies.

## Sidebar

`Sidebar` presents a flat set of labeled destinations with controlled semantic
selection:

```rust
Sidebar(
    "main-navigation",
    [
        SidebarItem::new("inbox", "Inbox"),
        SidebarItem::new("settings", "Settings").disabled(true),
    ],
    self.selected_destination_binding(),
)
```

Selection is `Binding<Option<String>>`. Item IDs must be nonempty and unique;
they remain the selection identity when items are reordered or labels change.
Use `.localized(message_id)` for a catalog label. A selected ID that is absent
from the current items produces no active row and remains in the owner binding
until the caller changes it. Disabled items retain the upstream disabled state
and have an explicit callback guard. The adapter uses the pinned GPUI Components
Sidebar and flat SidebarMenu; its outer native row paths remain position-based
and are not the RustUI semantic IDs. The wrapper uses `.w_full()` and disables
collapse, so it fills the width offered by its parent; there is no RustUI width
setter. Bound the parent when the sidebar needs a particular column width.

The sidebar is a tab stop. While its focus handle itself is focused, unmodified
Up/Down moves to the adjacent enabled item without wrapping; if selection is
absent or points at a disabled item, Up starts at the last enabled item and
Down at the first. Home and End select the first and last enabled items.
Enter runs a current-owner binding update that preserves the selected ID; the
control exposes no separate activation callback. The handler does not intercept
character input or modified keys. The tree role is named by the localized `rustui-navigation-tree` message by default, with `Navigation` as its fallback. Use `.accessibility_label("Settings navigation")` for a literal override or `.localized_accessibility_label("settings.navigation")` for an app message key. No active-descendant reference is emitted for the selected row.

M37 native inspection observed AX selected state, preservation across reversal
and relabeling, disabled pointer suppression, no active row on removal,
restoration, and Inbox selection. It did not accept the added keyboard behavior
or width behavior. Keyboard, focus, and full screenreader acceptance remain
pending. Evidence:
`target/headstart/milestone37-sidebar-gui.json`.

## NavigationSplit

`NavigationSplit` composes a controlled destination sidebar with the selected
detail in a horizontal, resizable two-column `SplitView`:

```rust
NavigationSplit(
    "settings-navigation",
    [
        NavigationSplitItem::new("profile", "Profile", Text("Profile detail")),
        NavigationSplitItem::new("privacy", "Privacy", Text("Privacy detail"))
            .disabled(true),
    ],
    self.destination_binding(),
)
```

The binding is `Binding<Option<String>>`; the caller owns it. Destination IDs
must be nonempty and unique and remain the selection identity when destinations
are reordered or relabeled. Disabling a destination prevents sidebar input from
changing selection; an external binding may still select that ID and render its
detail. `None`, or an ID with no current destination, renders an empty detail
and leaves the bound value unchanged.

Only the selected detail is lowered. Switching destinations releases the
inactive detail's retained native controls; values held in caller-owned bindings
remain with their owner. The composition uses GPUI Components' sidebar and
resizable split layout, with a 240-point initial sidebar width and a 140–480
point width range. Below 560 logical pixels it presents a single-column destination list/detail flow; see `docs/tasks/NAVIGATION_SPLIT_ADAPTIVE_COLLAPSE_DECISIONS.md`. Place
the split in a parent with bounded width and height so both columns receive a
finite layout extent. The M78 native checks used a 560×300 parent.

The navigation tree uses the localized `rustui-navigation-tree` message by
default (`Navigation` fallback). Override it with
`.accessibility_label("Settings navigation")`, or use
`.localized_accessibility_label("settings.navigation")` for an app message key.
Both override forms resolve on render; native screen-reader acceptance remains
open. The selected detail region uses that resolved
destination label as its accessible name.

The 560×300 native checks observed that `None` and unknown IDs hide the detail
while keeping the sidebar available; selecting Settings or Files from the
sidebar restored a detail. Files text and counter bindings updated, reversing
destinations preserved Files selection, and removing/restoring the selected
Files destination left the owner selection and bound values intact while
keeping the sidebar usable. This resolves the earlier whole-split collapse seen
with an unbounded gallery parent. Resize dragging, keyboard navigation, locale
switching, full accessibility review, and direct native allocation-release
instrumentation remain pending. Evidence:
`target/headstart/milestone78-navigation-gui.json` and
`target/headstart/milestone78-navigation-final.png`.

## Stack macro grammar

`VStack!` and `HStack!` accept adjacent constructor or nested-stack calls, optional
commas or semicolons, and chained builder calls. A bare second argument in
`Button(label, handler)` is rewritten to the owning component action. Conditional,
repeated, keyed, tuple, and arbitrary block content are not implemented yet.

## Application entry point

`RustUI` is a thin startup adapter over GPUI. It initializes GPUI Components,
creates the root component entity, mounts it in the GPUI Components root, opens a
window, and activates the application.

`RustUI::run` does not select or activate a RustUI theme. A root that uses
theme-dependent components, such as `Counter`, must activate a caller-selected
theme in `on_launch`, before RustUI mounts the root:

```rust
fn main() {
    let theme = rustui::theme::parse_theme(include_str!("../themes/macos27.json"))
        .expect("valid RustUI theme");

    RustUI::new()
        .on_launch(move |cx| {
            rustui::theme::activate_theme(&theme, None, cx)
                .expect("RustUI theme activation succeeds");
        })
        .run(Counter::default());
}
```

Choose the theme document and appearance that fit the application. Use
`.window_options(gpui::WindowOptions)` for advanced window configuration and
`.on_launch(|cx| ...)` for setup that needs GPUI's `App`. The launch hook runs
after GPUI Components initialization and before root creation, so the caller's
theme activation is in place when the root renders. GPUI continues to own the
application and window lifecycle.
`rustui::App` re-exports `gpui::App` for launch setup and the `View::on_mount` /
`View::on_unmount` lifecycle hooks, so application code can name those callback
types through RustUI without adding its own GPUI import.

## Localization activation

Parse catalogs with `Catalog::parse`, register them in `Catalogs`, then activate
the collection from the application launch hook:

```rust
let locale = Locale::parse("en").expect("valid locale");
let mut catalogs = rustui::localization::Catalogs::new(locale.clone());
catalogs
    .insert(rustui::localization::Catalog::parse(
        locale.clone(),
        include_str!("locales/en/app.ftl"),
    ).expect("valid catalog"))
    .expect("locale registered once");

RustUI::new()
    .on_launch(move |cx| {
        rustui::localization::activate_localization(catalogs, locale, cx);
    })
    .run(App::default());
```

`LocalizedText("message-id")` resolves during view lowering. A locale control can
use `Button("Français", set_locale_action(locale))`; the resulting `AppAction`
updates the GPUI global and GPUI Components locale. Missing messages render their
identifier after exact, primary-language, and configured fallback lookup.

For parameterized runtime text, pass named arguments to `LocalizedText`:

```rust
use rustui::localization::MessageArguments;

let arguments = MessageArguments::new().with("name", "Ada");
LocalizedText("greeting").with_arguments(arguments)
```

The message is resolved from the active locale and catalog fallback before
interpolation. `RenderContext::localized_message_with_arguments` provides the
same operation to custom views and returns `InterpolationError` when an
argument is missing or malformed. `LocalizedText` displays a visible
localization diagnostic if interpolation fails. The low-level formatter is
also available when formatting a previously resolved template:

```rust
let template = catalogs.resolve(&locale, "greeting");
let arguments = MessageArguments::new().with("name", "Ada");
let greeting = rustui_i18n::interpolate(template, &arguments)?;
```

The formatter accepts text, integers, and `f32`/`f64` decimal arguments, and
requires every referenced argument. Use `{{` / `}}` for literal braces. The
low-level `interpolate` function keeps numbers locale-independent;
`interpolate_with_locale` groups integer and decimal output for English, French,
and Arabic, including each locale's decimal separator and Arabic digit override.
Floating-point output uses shortest round-trip text without rounding. This is
number display formatting, not currency formatting or full CLDR number support.

`Locale::parse` normalizes language/script/region casing and accepts extension
and private-use subtag sequences such as `en-US-u-ca-gregory` and
`en-x-company-variant`. Variant subtags must be unique case-insensitively.
Extension singletons must be unique and have a value; private-use tags must have
at least one subtag. This is syntax-level handling, not validation against the
IANA language-subtag registry.

## Theme activation

`activate_theme(&ThemeDocument, mode, cx)` validates and applies the complete runtime
document. Button metrics and typography come from the typed `components.button`
record; the active result is stored in GPUI application globals and read by RustUI
Button during rendering. `apply_profile` remains the lower-level platform-palette
adapter. Theme activation does not claim native Liquid Glass blur.

`rustui::theme::ActiveTheme::mode` exposes the effective appearance from the most
recent successful activation. Playground's Theme and Layout story can opt into
350 ms theme-data polling with **Start auto**; it reads the file and applies a
validated change on the GPUI app thread, keeps the last valid theme on errors,
and stops on **Stop auto** or story unmount. Set
`RUSTUI_THEME_RELOAD_PATH` before launching Playground to use an isolated theme
file for this story. M65 observed valid edits applying, invalid JSON preserving
the last good theme, repair recovery, outer appearance selection remaining in
control, disabled polling, and no later activation after leaving the story. This
reloads theme data only; it does not hot-reload Rust source or serialize
view/state. It is not a filesystem watcher. Exact activation counts, unchanged
poll quiescence, large-file latency/CPU, missing-file/permission cases, and
multi-root isolation remain unverified. See `docs/tasks/AUTO_THEME_RELOAD.md`.

`theme::serialize_theme(&ThemeDocument)` validates a typed theme document and
returns normalized, pretty-printed JSON or `ThemeSerializeError`. `theme::resolve_button_with_selector(&ThemeDocument, &ButtonStyleSelector)` resolves Button rules for an explicit typed variant selector; `theme::resolve_button` keeps the appearance-only default behavior. This is a
document export; it does not preserve source bytes or serialize view trees,
component state, caches, or GPUI entities. The M26 workspace run passed 82 tests
across 12 binaries; see `target/headstart/milestone25-26-tests.log`.

M63–M65 integration validation passed 200 tests across 12 binaries, a build with
all three pinned GPUI dependencies fresh, and the locked/offline all-targets
check. Evidence: `target/headstart/milestone63-65-final-tests.log`,
`target/headstart/milestone63-65-reuse-report.json`, and
`target/headstart/milestone63-65-all-targets-check.log`. The shared native
bundle SHA-256 is
`e917a357fbe391eb428d0cb029d87fecb41cf7aba6e9dc128804618a4dd5d678`.

## Contract rules

Every supported public component contract must define:

- ownership and stable identity;
- mutation, binding and invalidation behavior;
- focus and keyboard behavior;
- accessibility semantics;
- theme tokens and supported variants;
- locale, logical direction and text behavior;
- async and lifecycle behavior where applicable;
- failure and disabled states;
- a reachable RustUI Playground story and validation evidence.

GPUI and GPUI Components remain implementation sources. RustUI does not expose an
Allui compatibility namespace or Allui types.

## Retained control lifetime (R2)

The host reuses a control's GPUI entity while its explicit identity and composition
scope remain stable. The same local control ID can be used in different tabs or
navigation pages. Two retained controls using the same ID in the same active scope
are rejected; reusing an existing retained ID for a different control type is also
rejected.

- Inactive tabs retain controls while their tab remains declared.
- Covered navigation pages retain controls while their route prefix remains in
  the path and their destination remains declared. The root is retained while covered.
- Popped/replaced pages, removed tabs, and controls absent from rendered content
  release the host's retained entity/subscription ownership.
- Bound root values survive this release. Recreated controls initialize from bindings;
  control-local state such as selection or undo history may reset.

Use stable explicit `.id(...)` values for dynamic `TabView`/`NavigationStack`
containers and distinct IDs for siblings. The tab default depends on count and
first tab; the navigation default is `navigation-stack`. Retention scopes are private.
Inactive subtrees reconcile internal structural changes when shown again. Dropping
registry ownership does not guarantee immediate destruction if GPUI holds references.
This description-only API adds no lifecycle callbacks. Explicit `Component` hosts
now provide child lifecycle; see the retained-child section below.

## Canvas drawing (M61)

`Canvas`, `CanvasSize`, `CanvasScene`, and the drawing command types are exported
through the public prelude. A canvas is a bounded drawing leaf; its fixed size
uses finite, nonnegative logical pixels. `CanvasScene::builder()` currently
supports filled rectangles, rectangle borders, and straight lines. Invalid
geometry or color input returns `CanvasError`; scenes preserve command paint
order and clones share immutable commands. Local drawing coordinates are clipped
to the canvas bounds. GPUI owns layout and painting; RustUI adds no drawing
runtime or scheduler.

```rust,ignore
let mut drawing = CanvasScene::builder();
drawing.fill_rect(CanvasRect::new(0.0, 0.0, 120.0, 80.0)?, color)?;
Canvas(CanvasSize::new(120.0, 80.0)?, drawing.build())
```

This slice does not include arbitrary paths, circles, text/image drawing,
per-shape hit testing, or animation. Canvas has no per-primitive accessibility
representation; wrap illustrations in `Semantics` and provide equivalent
information in regular RustUI content. The Playground story includes that
summary and equivalent text. Bounded M61 native review observed the basic
filled/outlined rectangles and line, clipping at both canvas sizes, clip-toggle
motion, and descriptive accessibility group labels. Evidence:
`target/headstart/milestone61-canvas-gui.json`, bundle SHA-256
`ba2979d96f69ed94c6dc1edd1fba285c65e529d452f6154c5016acaf79f5ebce`. Exact
pixels, scale-factor variation, screen-reader speech, per-shape accessibility,
and performance remain unverified. The repaired suite passed 190 tests across
12 binaries. See `docs/tasks/CANVAS.md`.

## ScrollView axes (M62)

`ScrollView(content)` and its concrete `ScrollViewView<V>` type are exported by
`rustui::layout` and `rustui::prelude`:

```rust
ScrollView(
    VStack::new()
        .child(Text("First section"))
        .child(Text("More content")),
)
.id("settings-content")
.axis(ScrollAxis::Vertical)
.height(320.0)
.scrollbars(true)
```

`ScrollAxis` supports `Vertical` (default), `Horizontal`, and `Both`. The default
fills available bounded space, scrolls vertically, and shows no scrollbar.
`.height(points)` preserves the prior behavior; `.width(points)` is new. Both
dimension requests must be finite and positive. `.scrollbars(true)` enables the
pinned upstream overlay bars; visibility and appearance follow upstream theme
policy, and bars reserve no layout space. Content does not shrink in scrolling
dimensions and is not virtualized.

The viewport, native scroll handle, content identity scope, and optional overlay
bars are retained together. Axis changes clear newly disabled coordinates while
preserving enabled offsets subject to native clamping. Single-axis modes use the
upstream gesture-axis lock; `Both` follows GPUI's dominant-axis policy. Scroll
space must be bounded by the parent or explicit dimensions. This adds no public
GPUI handle, programmatic scrolling API, or custom scrollbar implementation.

M62 has bounded native acceptance. The Both pane reported content size 1100×560
and maximum offset (580, 360); vertical wheel input moved its rows, and dragging
its horizontal scrollbar preserved the vertical offset. Parent updates and bar
toggles preserved offsets, axis changes cleared newly disabled coordinates, and
content shrink clamped offsets. Bound text survived pane removal/restoration;
Arabic translation and logical order were observed. Native listeners also
propagate wheel input to ancestors, matching upstream behavior; no containment
policy was added. Evidence: `target/headstart/milestone62-scroll-gui.json`,
bundle SHA-256 `8ead905f2c20ddc003372a7a509627f54df761fd38f622765bcde80412d6f8ba`.

The suite passed 191 tests with 0 failures across 12 binaries. Horizontal wheel
input produced delta zero, so real horizontal trackpad input, standalone and
nested horizontal gestures, shell sidebar/reference-pane wheel regressions,
diagonal scrolling, and full keyboard/screen-reader accessibility remain
unverified. Do not infer platform-wide acceptance from this review. See
`docs/tasks/SCROLL_AXES.md` and `apps/rustui-playground/src/stories/scroll_gallery.rs`.


## Keyed and optional composition (Milestone 7)

```rust
use rustui::prelude::*;

let rows = vec![("alpha", "Alpha"), ("beta", "Beta")];
let show_note = true;
let content = VStack::new()
    .children(rows.into_iter().map(|(id, label)| Keyed(id, Text(label))))
    .child(show_note.then(|| Text("Optional note")))
    .spacing(8.0);
```

`Keyed(key, view) -> KeyedView<V>` scopes both retained controls and GPUI element
identity without adding a layout box. Keys must be stable and unique within the
current logical retention scope, including stateless views. Duplicate keys panic
when lowered. An unkeyed stack does not introduce a scope; separate collections
can use distinct outer `Keyed` containers. Local control IDs may repeat across
separate keyed scopes.

`Option<V>` implements `View`: `Some` renders its child; `None` uses GPUI's
`Display::None` empty element and adds no stack gap. `VStack::children` and
`HStack::children` eagerly append an `IntoIterator<Item = V>`; they do not virtualize
rows. Different concrete view types require explicit type erasure such as `AnyView`.

Reorder retains controls under stable scope paths. Removing a visible row prunes
its retained entities/subscriptions; restoration recreates controls from bindings.
Root-bound values remain owned by the root. `Keyed` does not create independent
component state, action ownership, or lifecycle callbacks. Focus behavior has not
been accepted in the GUI.


## Retained child components (Milestone 8)

```rust
// Counter implements View and Default, with its own #[component] fields.
VStack::new()
    .child(Component("first-counter", Counter::default))
    .child(Component("second-counter", Counter::default))
```

`Component(key, initializer) -> ComponentView<C>` is an explicit stateful child
boundary. The initializer is `Fn() -> C + 'static`; it runs when a slot is first
created, restored after removal, or replaced with a different concrete component
type. A retained child keeps its current state across parent renders and reorder.
Initializer captures are initial values, **not live props**; changing those captures
in a new parent description does not update an existing child.

Each child has its own GPUI `ComponentHost`, retained controls, action/binding target,
and lifecycle callbacks. `on_mount` runs once per entity creation; `on_unmount` runs
when GPUI actually releases that entity. Parent registry removal releases its
reference; other GPUI/frame references can delay destruction. R2 declared inactive
tabs and covered navigation pages also preserve their child entities.

Keys must be unique within the current retention scope, even when two child
constructors use different concrete types. Unkeyed stacks introduce no scope.
Child slots are separate from control slots. The native GPUI entity ID provides
layout-transparent element identity, and replacing the component type produces a
new entity ID.

A plain `View`, `AnyView`, or `Keyed` remains a description, with no independent
owner. Child environment values are inherited at creation and refreshed when the
parent lowers that child again; see scoped environments below. Explicit live props
are available through `.props(...)`, described below.

Actions and bindings are relative to their rendering owner. Construct them inside
the child's `body`; do not pass a parent's lenses/actions across this boundary.
Wrong concrete owner types now panic with a diagnostic instead of silently doing
nothing. `Binding::read_from` still returns `None` as a non-mutating type probe.
Same-type instances cannot be distinguished by these type-erased lenses; there is
no parent-target action/binding API in this baseline. `AppAction` remains available
for explicitly application-level actions.

Compilation is complete; native focus, lifecycle timing, and interaction acceptance
have not been performed for this slice.


## Live child props (Milestone 9)

```rust
// ChildComponent and apply_title are implemented in the Playground's child_component.rs.
Component("alpha", ChildComponent::default)
    .props(String::from("Updated Alpha"), ChildComponent::apply_title)
```

`ComponentView<C>::props(props, reconcile) -> ComponentPropsView<C, P>` takes
`P: Clone + PartialEq + 'static` and `reconcile: fn(&mut C, &P)`. A noncapturing
closure or component method can serve as the reconciler. It explicitly assigns
incoming fields; child-local fields are preserved unless the callback writes them.

Initial props are projected before `on_mount`. A retained child's later props are
compared with its previous incoming value; unequal values invoke the reconciler
and notify the child through GPUI. Equal values do neither. Comparison follows
`P::PartialEq`, so equality must represent all inputs that affect reconciliation.
The callback must remain a stable projection for a given keyed component: swapping
callbacks with equal props does not trigger an update. Child-local edits alone do
not cause the same incoming props to be reapplied.

Changing props type while retaining the concrete child type applies the new props
without resetting the child. A different child type still replaces its entity.
Using plain `Component` at the same slot stops prop reconciliation and clears its
comparison baseline; it does not undo the last projected fields. Reattaching props
therefore applies them again. Dropping a child also drops its comparison metadata.

Inactive retained tabs/pages reconcile their latest props when lowered again.
Initializer captures remain creation-only. Inherited environment snapshots refresh
on parent rendering, as described below. Cross-owner bindings/actions are not added by this API.
The Layout story demonstrates parent-updated titles with child-owned counters and
notes. Its native interaction behavior is not yet accepted.


## Scoped environment (Milestone 10)

```rust
#[derive(Clone)]
struct DisplayMode(&'static str);

Environment(
    DisplayMode("detailed"),
    EnvironmentReader(|mode: Option<&DisplayMode>| {
        Text(mode.map_or("default", |mode| mode.0))
    }),
)
```

`Environment(value, view) -> EnvironmentView<T, V>` requires `T: Clone + 'static`.
It overrides that value's type for its subtree. Nested overrides shadow the same
type, while siblings and outer views retain their previous values. It introduces
no layout box, state owner, or identity scope; use `Keyed` to separate identities.

`EnvironmentReader(reader)` calls a typed `Fn(Option<&T>) -> V` during lowering.
Its returned view must own its data; borrowed environment references cannot escape.
Missing values are `None`. The reader retains the current host's action/binding
owner. Supplying custom environment data does not automatically switch RustUI
locale, theme, or platform settings.

Retained children receive the current inherited snapshot before native GPUI entity
lowering on a parent render. Removing an override also removes it from the next
inherited snapshot. Child state, props and lifecycle identity remain intact.
Inactive retained tabs/pages receive the latest snapshot when lowered again.

The update uses the current uncached GPUI child render path, with no unconditional
notification while the parent renders. There is no fine-grained environment
observation, automatic notification for interior mutation, or cached-child
invalidation contract. `EnvironmentValues` clones its map of shared values for a
scope/child snapshot; no performance benefit or negligible cost is asserted.


## Playground layout support and application boundary

RustUI Playground uses the public `rustui` API for its UI and directly depends on
`gpui-kit-assets` to register the default icon bundle. It does not use GPUI or
GPUI Components APIs directly. `Container(view)` supplies
an explicit flex-column layout box with width/height, padding, fill/grow sizing,
border, radius and semantic `Surface::{Background, Sidebar, Panel}` styling. It
inherits the active theme's foreground and font size. Unlike identity-only wrappers,
Container intentionally participates in layout. Dimensions must be finite and
positive; padding/radius must be finite and nonnegative.
`Container(view).opacity(value)` sets fixed opacity on its box and child. Values
must be finite in `0..=1`; without a call, no opacity override is set. The M45
native comparison showed its `1.0` sample brighter than its `0.5` sample across
the label and surface. Pixel alpha and child hit testing were not measured
(`target/headstart/milestone45-static-opacity-gui.json`).

`VStack` and `HStack` now expose `.fill()` and `.grow()` for bounded native flex
layouts. `RustUI::window_size(width, height)`, `.title(text)` and `.blurred_window()`
configure the native window through the framework. The Playground application uses
these APIs, `View`, state/bindings, components and public theme/localization APIs;
GPUI `Render` implementations and raw element construction remain inside RustUI.

## Explicit owner callbacks and background tasks

`OwnerActionReader(|owner: OwnerAction<Parent>| ...)` supplies a callback target for the exact mounted parent. Pass this target to child props; `owner.action(Parent::increment)` dispatches to that parent through GPUI. Targets compare by owner identity and keep only a weak update path.

`OwnerTaskReader(|tasks: OwnerTask<Parent>| ...)` supplies a task target for the exact mounted owner. `tasks.start(id, work, apply)` replaces work under that ID and reports `AsyncState::{Loading, Success, Error}`. `tasks.start_with_executor(id, work, apply)` additionally supplies the owning GPUI background executor to the work factory for executor-native delays or other async work. Both preserve the same replacement, cancellation, stale-result, and owner-release lifecycle. `tasks.cancel(id, apply)` reports `Idle`. The GPUI host retains task handles, and per-host generations reject stale completion after replacement or cancellation. Work factories must return a Send future; avoid blocking the UI while constructing it. Native start/restart/error/cancel checks and owner-release result suppression were exercised in M16/M25; blocking-work interruption is not guaranteed (`target/headstart/milestone25-owner-release-gui.json`).

`Spacer()` grows on the parent flex axis; `min_length(SpacerAxis, value)` requires a finite nonnegative value. `Divider()` is horizontal; `.vertical()` uses the upstream semantic separator. `LogicalInsets` resolves start/end using the current layout direction; bidi isolation helpers are opt-in and do not provide text shaping or translations. `ThemeFile` parses and validates reloads before replacing its last successful document.

## Native Grid and overlays

`Grid::new(NonZeroU16)` composes uniform fixed columns, with `.spacing`, `.child`, `.children`, `.fill`, and `.grow`. Source order advances across logical start-to-end columns and then subsequent rows. This is eager layout, not virtualization. Placement exceeding upstream grid-line limits is rejected.

`Group::new().child(view).children(iter).group(other)` builds an ordered child
list for insertion into `HStack`, `VStack`, `Grid`, or `ZStack` with `.group(...)`.
The receiving layout renders the group's children as its own children; the group
adds no GPUI element, layout item, gap, or identity scope. Nested groups flatten
in source order. `Group` is not a `View`, so it cannot be returned as a component
body or passed to a single-child wrapper such as `Container` or `Frame`. Use
`Keyed(stable_id, child)` for dynamic grouped content whose retained identity
must survive reordering. M63 observed matching qualitative placement against
direct siblings and retained keyed state across reorder, removal, and
restoration. Exact pixels, macro grammar extension, RTL matrix, native undo,
and full accessibility remain unverified. See `docs/tasks/GROUP.md`.

`ZStack::new()` overlays children in insertion order (last above first), centered by default. `.alignment(ZStackAlignment::new(horizontal, vertical))` selects logical alignment. Native screenshot acceptance currently covers the centered case. Both layouts reuse GPUI rather than introducing a second layout engine.

## Background and Border modifiers (M64)

`Background(id, child, surface)` and `Border(id, child)` decorate arbitrary
RustUI views with stable retained identity. IDs are mandatory and unique among
siblings of the same modifier family; Background and Border use separate typed
namespaces and may reuse an ID across families. Changing decoration values
preserves the child identity. Each modifier adds one native flex-column layout
box and no padding, clipping, event handling, or independent runtime.

`Background(...).radius(value)` rounds corners without clipping descendants.
`Surface::{Background, Sidebar, Panel}` map to the active palette's background,
sidebar, and secondary colors. Border defaults to one logical pixel, square
corners, and the active border color. `.width(value)`, `.radius(value)`, and
`.color(BorderColor::{Border, Accent, FocusRing})` select geometry and palette
entries; FocusRing is decorative and does not request or track focus. Dimensions
must be finite and nonnegative. Border width participates in box layout instead
of drawing an overlay. M64 observed independent same-ID editors across sibling
wrappers, retained values through styling/theme/RTL updates, root-bound values
through removal/restoration, and a bordered-button action. Exact pixels, layout
impact, native undo, full keyboard behavior, and screen-reader speech remain
unverified. See `docs/tasks/VIEW_DECORATION.md`.

## Native focus scope

`FocusScope(id, focused: Binding<bool>, child)` retains one native focus handle
in a scope identified by a stable, nonblank sibling-unique ID. The binding acts
as both a focus request and a report while the scope is rendered: false→true
requests focus on the scope container if it or a descendant is not already
focused; true→false blurs only while focus remains inside. Native focus-in/out
updates the binding only when its value changes. The wrapper is a native Group;
`.label(...)` names it and `.tab_stop(true)` includes the container in tab
traversal (default false).

```rust
FocusScope(
    "search-scope",
    self.search_scope_focused_binding(),
    TextField("Search", self.search_binding()),
)
.label("Search scope")
```

M52 bounded native interactions covered two scopes, descendant text entry,
pointer focus, conditional blur and child removal/restoration. A transient
both-scopes-true report appeared while native focus was settling. Removing a
focused scope does not clear the caller's binding; callers should clear it if
the scope is removed. Programmatic focus targets the scope container, not a
private handle inside an opaque child control. Optional scope tab-stop and full
keyboard/screen-reader acceptance remain open. See
`docs/tasks/FOCUS_SCOPE.md` and
`target/headstart/milestone52-focus-scope-gui.json`.

## Controlled CodeEditor

`CodeEditor(id, Binding<String>)` retains the upstream GPUI Components
`EditorState`; keep a stable nonblank ID associated with the same owner binding
field. Defaults are plain-text language, 240 logical points high, and visible
line numbers. `.language`, `.placeholder`, `.line_numbers`, `.height`,
`.disabled`, `.readonly`, `.tab_index`, and `.accessibility_label` configure the
native editor. Binding updates and configuration changes reuse its retained
entity; removal releases that native editing state.

```rust
CodeEditor("source", self.source_binding())
    .language("rust")
    .line_numbers(true)
    .height(240.0)
    .accessibility_label("Document source")
```

M53 bounded native editing observed binding synchronization, language and gutter
changes preserving text, readonly/disabled input suppression, external
replacement, and source remount. The workspace disables upstream parser
features, so language settings do not establish syntax highlighting; adding
the parser feature still requires approval. There is no LSP integration.
Undo coalescing and history across removal/remount are not guaranteed, and full
editor accessibility/screen-reader acceptance remains pending. See
`docs/tasks/CODE_EDITOR.md` and
`target/headstart/milestone53-code-editor-gui.json`.

## Virtualized LazyGrid

`LazyGrid::new(id, items, columns, viewport_height, row_height, factory)` uses
GPUI `uniform_list` to virtualize rows. `items` is an `Rc<[LazyGridItem<T>]>`;
each item needs a stable, nonblank ID, unique across the full snapshot.
`columns` is `NonZeroU16`; viewport and row heights must be finite and positive.
`.column_gap` and `.row_gap` accept finite nonnegative values, with combined
dimensions checked for overflow. Columns have equal width and are not
virtualized. The factory runs for rows requested by GPUI, including measurement
ranges. Rows follow logical start-to-end order and mirror in RTL.

```rust
LazyGrid::new("catalog", items, NonZeroU16::new(3).unwrap(), 260.0, 90.0,
    |index, _value| Text(format!("#{index}")))
    .column_gap(8.0)
    .row_gap(6.0)
```

M54 native acceptance showed painted three-column rows, stable child state while
scrolling/reordering/prepending, fresh child state after removal/restoration,
release of visited owners when hidden, and Arabic placement with a live French
locale update. Factory counts include measurement and repeated render requests;
they do not establish exact overscan or speed. Retained cell state grows with
visited IDs still in the current dataset. Native row IDs are positional;
keyboard/screen-reader coverage, background-task/environment propagation, and
stale detached-callback behavior remain unverified. See
`docs/tasks/LAZY_GRID_IMPLEMENTATION.md` and
`target/headstart/milestone54-lazy-grid-gui.json`.

## Frame modifier

`Frame(child)` adds native size constraints and alignment without a theme
surface, border, or clipping. `.width`, `.height`, `.min_width`, `.max_width`,
`.min_height`, and `.max_height` accept finite nonnegative logical points.
Contradictory min/max pairs panic; a fixed size beyond a bound is passed to
native layout and clamped there. Natural dimensions default to centered child
alignment. Logical horizontal Start/End follow the current locale direction;
Stretch and Baseline delegate to GPUI alignment behavior. Frame creates no new
retained view owner.

```rust
Frame(Text("Content"))
    .width(240.0)
    .max_width(160.0)
    .height(48.0)
    .alignment(HorizontalAlignment::Start, VerticalAlignment::Center)
```

M55 bounded visual acceptance observed LTR and Arabic Start/End reversal,
centered content, a visibly narrower 240/max-160 sample, zero-width without a
crash, and a framed button whose count persisted through a locale switch. These
were qualitative observations, not pixel measurements.

M74 later verified a stateful single-child example: Center retained typed text,
Stretch expanded the background to the Frame bounds without losing the value,
Baseline rendered without crashing and retained text, Command-Z restored prior
text, returning to Center kept the value, and the framed button count changed
0→1. A stable inner Frame width of 160 with horizontal Stretch resolved a
percent-width Input holder that had collapsed in earlier attempts. No peer
baseline-placement claim applies to the single-child sample. Exact pixels,
caret/selection, full keyboard/AX/screen-reader, RTL, and cross-platform
acceptance remain unverified. The M74 workspace suite passed 238 tests, 0 failed
across 12 binaries (Cargo exit 0); its locked/offline all-targets check exited 0.
Evidence: `docs/tasks/FRAME_MODIFIER.md`,
`target/headstart/milestone74-frame-gui.json`,
`target/headstart/milestone74-workspace-tests.log`, and
`target/headstart/milestone74-all-targets-check.log`.

The shared M54–M55 test suite passed 161 tests across 12 binaries; the
all-targets workspace check also passed. See
`target/headstart/milestone54-55-paint-tests.log` and
`target/headstart/milestone55-all-targets-check.log`.

## Typed same-process drag and drop (M66)

`DragSource(id, value, child)` and `DropTarget(id, binding, child)` wrap GPUI's
native drag source and typed drop listener. The source copies a `Clone +
'static` value into the active drag. The target accepts the same Rust payload
type and writes an owned `DropEvent<T>` through its component-owned binding.
The event contains source and target IDs, the payload, and a
`DragOperation::{Copy, Transfer}` intent. The application applies that intent
to its own model; RustUI does not remove source data or promise an atomic
cross-owner move. Disabled wrappers render their child without registering a
listener. Use `Keyed(stable_id, ...)` for dynamic collections.

This contract covers same-process pointer drag/drop. Bounded native pointer
acceptance recorded copy/drop, transfer, reordering, disabled source/target,
incompatible numeric payload, and target removal/restoration. Keyboard-only
acceptance and actual component-owner destruction during an in-flight drag
remain pending. OS file dragging, cross-window transport, touch input, external
payloads, and GPU optimization are outside scope. See
`docs/tasks/DRAG_AND_DROP.md` and
`target/headstart/milestone66-drag-drop-gui.json`.

## Native Shadow modifier (M67)

`Shadow(id, child)` applies the active GPUI Components theme's Medium elevation
and has its own stable retention scope. `.elevation(ShadowElevation::{Small,
Medium, Large})` selects a native theme elevation. `.style(ShadowStyle)` replaces
it with one explicit outer shadow; `.radius(value)` sets the shadow box radius
without clipping its child. `ShadowStyle::new(offset_x, offset_y, blur, spread,
color)` returns a validated style or `ShadowStyleError`: offsets and spread
must be finite, blur finite and nonnegative, and Hsla channels finite in 0..=1.
Negative offsets and spread remain valid native parameters. Explicit style color
does not follow the theme's disabled-shadow policy. No inset or custom GPU
shadow path is provided. Bounded native inspection observed Dark and Light
theme elevations, explicit style changes, independent retained fields,
omission/restoration, owner action, and Arabic RTL with values retained. Exact
pixels, a native shadow-disable toggle, full keyboard/AX speech, and performance
remain unverified; no full acceptance claim is made. See
`docs/tasks/SHADOW_MODIFIER.md` and
`target/headstart/milestone67-shadow-gui.json`.

## Themed Card (M68)

`Card(id, child)` composes a Panel `Container` with a theme-backed `Shadow`.
Defaults are 16 logical pixels of padding, 10 pixels of radius, and Medium
theme elevation. `.padding(value)` and `.radius(value)` accept finite
nonnegative logical pixels; `.elevation(...)` chooses Small, Medium, or Large;
`.border(enabled)` toggles a one-pixel theme border and defaults to off. Card
IDs must be nonempty and scope child retention. Radius does not clip children.
Card adds no focus behavior, accessibility role, action handling, or component
owner. Bounded native acceptance observed usable editable controls at the
gallery's equal 260-point card width, independent same-ID child values, custom
styles, omission/restoration, Light appearance, Arabic RTL, and English LTR
restoration. The measured width is gallery layout, not an intrinsic Card sizing
rule. Exact pixels/padding, cursor/undo, full AX speech/keyboard, and performance
remain unverified. See `docs/tasks/CARD.md` and
`target/headstart/milestone68-card-gui.json`.

## Conditional stack children (M69)

`VStack!` and `HStack!` accept `if`, `if`/`else`, and `else if` child branches
alongside existing child expressions. Branch children are flattened into the
receiving stack through `Group`; the branch itself adds no layout node or
retention scope. Conditions follow ordinary Rust short-circuit selection, and
unselected child constructors are not evaluated. Use explicit `Keyed` IDs for
stateful dynamic children.

The macro does not add `for`, `match`, arbitrary statement/block, tuple, or bare value child syntax. Existing `.children(...)`, `Option<V>`, `AnyView`, and builder APIs remain available for those cases. Bounded native acceptance is complete: branch changes, explicit keyed editor reorder, actions, parent updates, and optional-child removal/restoration were observed. Explicit `Keyed` identity is required for stateful children. RTL, full keyboard/accessibility speech, performance, cursor/undo, and release callbacks remain unverified. See `docs/tasks/CONDITIONAL_STACKS.md` and `target/headstart/milestone69-conditional-stacks-gui.json`.

## TapRegion pointer wrapper (M70)

`TapRegion(id, child, action)` wraps a view in a native GPUI click region.
`.enabled(false)` leaves the child rendered and skips listener registration.
Each region has a typed retention scope and distinct native ID namespace.
GPUI owns hit testing and bubble-phase click dispatch; TapRegion does not stop
propagation or prevent default behavior.

TapRegion does not create a focus target, keyboard activation contract, or button accessibility role. Provide a separate keyboard-accessible control for essential actions. Bounded native pointer acceptance is complete: bubbling, independent child inputs, disabled wrappers, alternative button actions, and removal/restoration were observed. Keyboard-only alternative-button activation, exact pixels, RTL, full accessibility speech, and performance remain unverified. See `docs/tasks/TAP_REGION.md` and `target/headstart/milestone70-tap-region-gui.json`.

## Typed PropertyInspector (M71)

`PropertyInspector(id, rows)` composes controlled `TextField`, `Toggle`, and
`Stepper` rows using `Binding<String>`, `Binding<bool>`, and `Binding<i32>`.
`PropertyInspectorRow::text`, `::toggle`, and `::integer` construct the typed
rows; row and inspector IDs must be unique where required and nonblank. Rows
retain caller order, and explicit row IDs keep state associated through reorder.
Inspector/row disabled state is passed to the wrapped controls.

Bindings must belong to the component owner rendering the inspector. V1 adds no
reflection, serialization, validation rules, nested groups, dynamic type
erasure, or cross-owner binding support. Bounded native acceptance is complete:
the Name input exposed an accessible name; primary and sibling `name` rows kept
independent edited values; toggle/integer bindings, reorder, disabled-row and
disabled-inspector guards, and restored editing were observed. Native cursor/undo,
inspector removal, full keyboard/accessibility speech, locale/theme coverage, and
performance remain unverified. See `docs/tasks/PROPERTY_INSPECTOR.md` and
`target/headstart/milestone71-property-inspector-gui.json`.

## M69–M71 verification status

The M69–M71 workspace suite passed 236 tests across 12 binaries, the build succeeded with all three pinned GPUI dependencies fresh, and the all-targets check exited 0. Bounded native acceptance is complete for M69, M70, and M71, with the limits recorded in their task contracts. The expanded test log is `target/headstart/milestone71-expanded-tests.log`. See the milestone task contracts and `docs/IMPLEMENTATION_ROADMAP.md`.


## State-driven translation

`SpringOffset::new(id, view, x, y)` uses two GPUI retained springs for finite
signed logical-pixel targets. The ID must be nonblank; blank or whitespace-only
IDs return `SpringOffsetError::EmptyIdentity`. Keep `id` stable across renders to
retarget each axis; newly mounted springs begin at their target. GPUI owns scheduling and
app-level reduced-motion behavior. The child keeps its layout slot; GPUI applies
the translation to native hit-test and accessibility bounds. Parent clipping,
overlap, stacking, and accessibility-tree behavior remain native. It does not
scale or rotate, and source-level tests do not establish intermediate frames,
pixels, velocity continuity, or performance.

## ResizablePanel

`ResizablePanel` is the public advanced-component name for the existing
`SplitPanel` descriptor. Use it inside a `SplitView`; it shares the same keyed
child ownership, finite initial-size/range validation, native resizing, physical
arrow-key behavior and RTL panel ordering. It does not mount independently.

```rust
use rustui::prelude::*;

let panels = SplitView::horizontal("workspace")
    .panel(ResizablePanel::new("sidebar", Text("Sidebar"))
        .initial_size(200.0).size_range(100.0..300.0))
    .panel(ResizablePanel::new("content", Text("Content")));
```

The canonical SplitView story uses this API, including stateful child panels,
reorder/removal/restoration, range changes and axis changes. The GPUI event test
exercises this public name through actual divider focus/key dispatch and native
resize state. Native/browser visual, AX and VoiceOver acceptance remain separate.
