# M1212 — Stage 0 clarify architecture guide acceptance wording

Updated the Playground status paragraph in `docs/ARCHITECTURE.md` to say that
full-application interaction, visual, accessibility, and GUI acceptance remain
open, while linking the bounded reviews recorded in
`docs/RUSTUI_PLAYGROUND.md`. The prior wording described all such acceptance as
pending despite completed focused reviews.

Verification: `git diff --check -- docs/ARCHITECTURE.md` passed. This was a
documentation-only correction; no tests were run.
