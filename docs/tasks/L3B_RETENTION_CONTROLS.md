# L3b Lab retention controls

The Tabs and NavigationStack stories now provide separate bound text editors for each tab or page. Each story reuses the local editor ID `retention-editor` in different tab or page scopes so manual inspection can check that control state stays isolated. The TabView and NavigationStack use stable container IDs.

## Manual exercise

1. In Tabs, enter different text in Overview and Details, switch between them, and check each value. Remove Details with the control above the canvas, then restore it. If Details was selected when removed, Overview becomes selected. The Details binding keeps its text while the tab is absent.
2. In Tabs, show the conditional Overview editor, enter text, hide it, then show it again. Its binding keeps the text while the editor is absent.
3. In NavigationStack, enter different text in root, Details, and Inspector. Navigate forward and back to inspect each page. Show and hide the conditional root editor and check its bound text.
4. Inspect the event log for remove, restore, show, and hide actions. The controls are visible only in their matching story.

Bound text belongs to Lab state and can persist when a tab, page, conditional editor, or entire story preview is removed. Cursor position and undo history belong to a control instance: the retention behavior is intended to preserve them for inactive declared tabs and covered pages, while removed tabs, popped pages, absent conditional editors, and story switches release those instances. Returning to a page does not promise restored keyboard focus.

This note describes manual exercise paths, not a GUI acceptance result. The Lab's fixed story list and unscrolled vertical layout can limit visibility at small window sizes.

## Integration

Root integration uses 80px editors and enables the conditional-editor controls
only on their visible Overview/root page. The current stack macro requires a
call-shaped child here; the prebuilt controls apply `.spacing(8.0)` at composition.
The final locked all-target workspace check passed (exit 0, 2.33 seconds), and
non-recursive formatting checks passed. No tests or GUI checks ran.

The agent's first rustfmt invocation traversed modules and updated
`locale_selector.rs`'s timestamp. No before-byte snapshot was available to establish
a content change; no speculative restoration was performed. Root formatting used
`--config skip_children=true` thereafter.
