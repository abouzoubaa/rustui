# M762 — Stage 8 / fall back on padded SplitView splitter key echoes

The SplitView splitter accessible name now treats whitespace-padded echoes of
its localization key as unresolved and returns the documented `Resize
divider` fallback.

The focused locked/offline
`splitter_name_uses_a_human_readable_fallback_without_localization` test passed
1/1; scoped `git diff --check` passed. Native accessibility-tree, VoiceOver,
locale-switching, keyboard-resize, and cross-platform acceptance remain open.
