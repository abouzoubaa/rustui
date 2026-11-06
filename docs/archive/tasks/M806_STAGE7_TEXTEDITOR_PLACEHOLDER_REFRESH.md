# M806 — Stage 7 TextEditor placeholder refresh

M921 stabilizes implicit TextEditor identity across placeholder localization,
so the retained-state refresh path below still runs after a locale switch.

`TextEditorView::into_element` previously applied its placeholder only when
creating the retained `TextareaState`. It now compares the retained placeholder
with the current owner value and calls `set_placeholder` when they differ.

The focused regression
`changed_placeholder_is_refreshed_in_retained_textarea_state` passed. This is
source-level synchronization evidence; native input, IME, and accessibility
acceptance remain open.
