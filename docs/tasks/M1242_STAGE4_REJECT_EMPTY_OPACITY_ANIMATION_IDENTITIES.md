# M1242 — Reject empty opacity animation identities

`AnimatedOpacity` rejects empty and whitespace-only animation identities with
`OpacityAnimationError::EmptyIdentity`. Nonblank identities remain unchanged.
The focused identity test filter passed (3/3; opacity, offset, and spring).

Native animation acceptance remains open; this milestone records a source and
focused-regression result only.

## 2026-10-24 current-checkout verification

The focused identity filter passed again, covering opacity, offset, and spring:

```sh
./scripts/cargo-headstart test -p rustui --lib rejects_blank_ids_and_preserves_nonblank_identity --locked --offline
```

Result: 3 passed, 0 failed. The full RustUI library suite also passed:

```sh
./scripts/cargo-headstart test -p rustui --lib --locked --offline --quiet
```

Result: 867 passed, 0 failed. Native animation acceptance remains open.
