# D1c Link preview specimen

The compiled preview gallery includes an opt-in Link specimen backed by the
public RustUI `Link` component. Its inspector stores only a nonempty label
(1–80 characters) and URL (1–2048 characters). The component emits the existing
empty-payload `activate` signal through the current generation-stamped preview
action contract.

The preview sets `open_url_on_activate(false)`, so activating the specimen
dispatches its local preview signal without handing the URL to GPUI's platform
opener. The `Link` builder defaults this option to `true`, preserving existing
application behavior. The URL is inert preview data and is never opened by
mounting, editing, importing, exporting, or activating this specimen.

The focused regression checks the bounded schema, empty payload, action
dispatch, and descriptor round trip. The Link builder regression checks that
platform opening defaults on and can be explicitly suppressed.

Validation used isolated target `target/headstart/d1c-link-validation`:

- `./scripts/cargo-headstart test --locked --offline -p rustui --lib components::basic::link::tests` — passed, 7 tests.
- `./scripts/cargo-headstart test --locked --offline -p rustui-playground --lib stories::compiled_preview_gallery::tests::link_specimen_keeps_bounded_label_and_url_and_dispatches_empty_signal` — passed, 1 test.
- `./scripts/cargo-headstart check --locked --offline -p rustui-playground --all-targets` — passed.
- `rustfmt --check --edition 2024` on the two changed Rust sources and scoped `git diff --check` — passed.

Logs are retained in `target/headstart/evidence/` with the `d1c-link-` prefix.
The initial focused gallery run caught a missing mount-time action mapping; the
fixture now installs its allowlisted action, and the rerun passes. The failed
first run is preserved as `d1c-link-preview-tests-initial-failure-20261020.log`.
