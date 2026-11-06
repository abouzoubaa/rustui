# Stage 9 / announce Playground owner-task status changes

The owner-task story displayed Idle, Loading, Success, and Error as ordinary
text. As asynchronous task state changed, assistive-technology users had no
live announcement for those updates.

The current state is now exposed through RustUI's existing polite status
live-region API. A focused regression covers the status text for all four task
states. This is source-level coverage; native accessibility-tree, spoken
announcement, and cross-platform acceptance remain open.

The focused regression passed 1/1:

```text
./scripts/cargo-headstart test -p rustui-playground --lib owner_task_status_text_covers_idle_loading_success_and_error
```

Scoped formatting and diff checks passed.
