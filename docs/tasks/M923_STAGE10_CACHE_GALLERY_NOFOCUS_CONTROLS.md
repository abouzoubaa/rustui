# M923 — Stage 10 / add no-focus controls to the cache gallery probe

The cache gallery now has keyboard shortcuts for the M117/P3B parent-only
probe: `Secondary-Shift-P` ticks the parent; `Secondary-Shift-C` clears
lowering counts; `Secondary-Shift-S` samples counters; and
`Secondary-Shift-A` samples accessibility separately. The gallery instructs
the operator to focus the diagnostics control once and keep focus there while
running the probe. This avoids using action buttons that would themselves
change focus and invalidate the comparison.

The focused parent-only regression passed 1/1 (140 filtered):

```sh
./scripts/cargo-headstart test -p rustui-playground --lib \
  stories::cache_gallery::tests::parent_only_tick_preserves_child_props_revision \
  --locked --offline
```

Scoped `rustfmt --check` and `git diff --check` passed. The worktree file
already contained M689's diagnostics-child extraction; that work was preserved.
The shortcut probe has not been run in a native window. M117 still requires
the no-focus cache-lowering observation and separate focus/accessibility
evidence.
