# N1: Navigation controls and unavailable destinations

## Edits

- `crates/rustui/src/components/navigation/navigation_stack.rs` renders NavigationLink and Back through the existing RustUI `Button` using `AppAction::new` and the owner-current `binding_updater`. The link keeps its explicit ID and disabled setting; Back keeps its derived ID. The matching RustUI button styles retain their former secondary and ghost appearances while using RustUI theme metrics and accessibility defaults.
- `locales/{en,fr,ar}/rustui.ftl` adds simple messages for Back and unavailable destination status. The route is rendered as separate text because the catalog parser accepts simple messages only.

## Behavior and API

An empty path shows the root. A known last route shows its destination. An unknown last route shows the localized unavailable status followed by the route string. The bound path is retained, and Back removes one route using the current owner value. Rendering never changes the path. Public constructors, IDs, and disabled behavior are unchanged. This is a visible fallback for missing destinations, not a new route resolver or navigation engine.

## Dependencies and limits

Navigation views → RustUI `Button` → `AppAction` → `RenderContext::binding_updater` → GPUI component owner. Locale messages come from `RenderContext::localized_message`, which returns the key if no active localization exists. Translation wording needs native-speaker review. GUI checks remain pending for button appearance, focus and accessibility announcements, missing destination text in all three locales, right-to-left layout, and Back behavior after rapid path changes.

## Acceptance status

Source change complete. No tests, Cargo build, or GUI run were performed in this task; the parent task owns shared compilation.
