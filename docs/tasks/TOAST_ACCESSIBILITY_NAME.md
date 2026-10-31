# Toast accessible-name validation

`ToastBuilder::show` requires either a nonblank title or message so the
notification has useful accessible text. A blank message remains valid when a
nonblank title is supplied. Toast action labels reject empty and whitespace-only
text because the label is also forwarded as the action button's accessible name.

Focused tests cover blank action labels, blank title/message combinations, and
the titled empty-message case. Validation passed:

- `./scripts/cargo-headstart test --locked --offline -p rustui --lib components::overlay::toast::tests` (6 passed)
- `rustfmt --edition 2024 --check crates/rustui/src/components/overlay/toast.rs`
- `git diff --check -- crates/rustui/src/components/overlay/toast.rs`

The test output is preserved at
`target/headstart/evidence/toast-accessible-name-20261020.log`. This is
source-level validation; it does not establish native accessibility-tree or
screen-reader acceptance.
