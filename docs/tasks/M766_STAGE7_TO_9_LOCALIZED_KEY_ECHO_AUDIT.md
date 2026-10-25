# M766 — Stages 7–9 localized key-echo fallback sweep

Closed remaining source-level accessible-name fallback gaps found in Button,
Avatar, Checkbox, ColorPicker, Icon, Image, Picker, Slider, NavigationSplitView,
Sidebar, Toolbar, Popover, and Alert's localized close action. Resolvers now
treat blank output or whitespace-padded message-ID echoes as unresolved and
preserve the existing explicit or visible-label fallback. Slider and Toolbar
now pass the message ID to their shared resolver; real translations remain
verbatim.

Scoped `git diff --check` passed for the changed source files. No tests, build,
or GUI acceptance were run in this coding pass. Native accessibility-tree,
VoiceOver, locale/RTL, keyboard, and cross-platform acceptance remain open.
