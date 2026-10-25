# M778 — Stage 8 lock down Popover padded-key fallback

Added a focused regression proving that a whitespace-padded localized
message-key echo is treated as unresolved and the Popover's visible trigger
label is used. The fallback behavior was already present in the shared source
changes; this milestone adds direct regression coverage for it.

The focused regression passed 1/1, and `git diff --check` passed for the
Popover file. `rustfmt --check` still reports pre-existing import-order
differences in that dirty file. Browser interaction, native accessibility,
and cross-platform acceptance remain open.
