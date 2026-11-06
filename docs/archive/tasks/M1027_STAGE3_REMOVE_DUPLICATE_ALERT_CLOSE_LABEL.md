# M1027 — Remove duplicate Close label from the Alert gallery

The Alert story rendered a standalone localized `rustui-action-close` label
next to an Alert that already supplies its own localized Close button. The
extra row duplicated the action label and made the specimen harder to read.

The redundant row was removed from `apps/rustui-playground/src/stories/alert_gallery.rs`.
The focused package check passed:

```text
./scripts/cargo-headstart check -p rustui-playground
exit 0
```

Native visual, keyboard, and full accessibility acceptance remain open.
