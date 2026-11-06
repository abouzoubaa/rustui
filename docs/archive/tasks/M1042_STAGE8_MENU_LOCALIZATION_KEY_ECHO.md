# M1042 — Stage 8 / fall back from unresolved Menu localization keys

Menu action localization previously treated an unresolved message-key echo as
a valid label. A key such as `menu.open` could therefore be exposed as the
visible and accessible action name when no translation was available. Menu
now detects blank results and whitespace-padded key echoes, then uses the
original action label as its fallback. Resolved translations remain verbatim.

The focused `unresolved_menu_translation_echo_uses_original_label` regression
passed 1/1 under the locked, offline Headstart test command; scoped rustfmt
passed. Native menu accessibility-tree, VoiceOver, locale-switching, and
cross-platform acceptance remain open.
