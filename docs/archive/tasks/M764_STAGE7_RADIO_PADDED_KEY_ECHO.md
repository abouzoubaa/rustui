# M764 — Stage 7 Radio padded localization key echo

Radio's accessible-name resolver trims both the localized value and message ID
when detecting an untranslated key echo. Explicit and visible-label fallbacks
remain in effect, and real translations are preserved verbatim.

The focused Radio localization regression passed (1 passed, 745 filtered), and
the scoped diff check passed. Native accessibility, VoiceOver, locale-switching,
and cross-platform acceptance remain open.
