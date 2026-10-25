# M763 — Stage 8 / fall back on padded TabView localization key echoes

TabView now treats whitespace-padded message-ID echoes as unresolved for both
the tab group's accessible name and an individual tab's accessible name. The
documented `Tabs` or visible-label fallback is used, and nonblank translations
remain verbatim.

The focused group-name and tab-name regressions passed 1/1 each under locked,
offline `cargo-headstart` tests; scoped formatting and diff checks passed.
Native accessibility-tree, VoiceOver, locale-switching, RTL, keyboard, and
cross-platform acceptance remain open.
