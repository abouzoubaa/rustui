# M1079 — Explain unsupported stack control flow

Keep `for` and `match` outside the supported stack-child grammar. When either
appears in a stack macro, report that callers can expand those branches before
the macro, so the diagnostic gives an actionable recovery path without
expanding the language.

The focused parser regression covers top-level `for` and nested `match`; it
passed 1/1. Broader Stage 4 grammar and Playground acceptance remain open.
