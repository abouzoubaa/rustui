# Stage 9 / announce delayed preview rerender failures

The compiled-preview gallery reports a failed delayed parent rerender in its
status text, but the polite live-region filter did not include this failure
message. The status could therefore be visible without being announced to
assistive technology.

The live-region filter now includes `Delayed parent rerender failed:` and the
focused regression covers that message. This is source-level coverage only;
native accessibility-tree, spoken announcement, and cross-platform acceptance
remain open.

The focused regression passed 1/1:

```text
./scripts/cargo-headstart test -p rustui-playground --lib delayed_rerender_changes_state_only_on_completion_or_error
```

Scoped `git diff --check` passed. A full-file rustfmt check reports pre-existing
formatting differences elsewhere in the shared file; those were left unchanged.
