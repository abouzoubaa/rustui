# M767 — Stage 9 / keep Playground owner-task delays nonblocking

The Playground owner-task success, restart, and failure examples now await
`BackgroundExecutor::timer` futures through `OwnerTask::start_with_executor`.
They no longer sleep a GPUI background worker thread while simulating delayed
results. The helper accepts an ordinary `Send + 'static` future, so the
Playground still depends only on RustUI's public API.

`rustfmt --edition 2024 --check apps/rustui-playground/src/stories/owner_tasks.rs`,
scoped `git diff --check`, and
`./scripts/cargo-headstart check -p rustui-playground --lib --locked --offline`
passed. No story interaction or GUI acceptance was run; those remain open.
