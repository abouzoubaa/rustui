# M765 — Stage 7 Stepper padded localization key echo

Stepper's accessible-name resolver trims both the localized value and message
ID when detecting an untranslated key echo. Explicit and visible-label
fallbacks remain in effect, including the increment and decrement action
labels, and real translations are preserved verbatim.

The focused Stepper localization regression passed (1 passed, 0 failed), and
the scoped diff check passed. Native accessibility, VoiceOver, locale-switching,
and cross-platform acceptance remain open.
