# M1051 — Stage 8 reject blank Link identities

`LinkView::id` now rejects whitespace-only identities, matching the nonempty
identity contract used by peer components. This prevents a blank explicit value
from being used as the rendered element identity in place of the stable
label-and-URL fallback.

Focused regression passed:

```text
./scripts/cargo-headstart test -p rustui --lib components::basic::link::tests::rejects_blank_explicit_identity -- --exact
1 passed; 0 failed
```

Broader Link behavior, GUI focus/activation, and screen-reader acceptance remain
unverified by this focused check.
