# M1016 — Stage 6: recognize the Syriac RTL default

`LayoutDirection::for_locale` now treats primary language `syr` (Syriac) as
right-to-left by default, while an explicit `Latn` script preserves left-to-
right behavior.

The focused regression for `syr` and `syr-Latn` passed 1/1.

Broader script/language coverage, native RTL GUI acceptance, and cross-platform
acceptance remain open.
