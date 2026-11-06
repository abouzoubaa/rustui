# M1082 — Classify the explicit Avestan script as RTL

When locale tags explicitly request script `Avst`, return right-to-left
direction even when the language is undetermined. A focused regression for
`und-Avst` passed 1/1.

This adds script classification only; broader localization, native layout,
and screen-reader acceptance remain separate.
