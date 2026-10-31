# M975 — Stage 6 / default Dari locales to right-to-left

Locale direction now includes Dari (`prs`) in the default RTL language set.
An explicit `Latn` script remains LTR.

Focused check: `dari_defaults_to_right_to_left_unless_script_is_overridden`
passed (1/1).

This is a bounded language classification, not full CLDR locale-direction
coverage.
