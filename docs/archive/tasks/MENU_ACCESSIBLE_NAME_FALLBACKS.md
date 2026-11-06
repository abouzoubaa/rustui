# Menu accessible-name fallbacks

## Contract

Menu action labels, localization keys, and trigger IDs must be nonblank. Blank resolved translations retain the literal action label; nonblank translations replace it. `PopupMenuItem::Item` passes the label to the outer menu-item role.

## Current implementation gap (2026-10-22)

M353 uses `ElementItem` to preserve RustUI `menu_item` typography and foreground, placing `.aria_label(label)` on the child label. In pinned GPUI Components 0.7.3, `ElementItem` exposes no outer accessible-label value, so source does not confirm the outer role receives the label. The child may still contribute a computed name. Switching back to `Item` would lose M353 styling; preserving both requires a separately approved pinned API extension. Native accessibility-tree and VoiceOver acceptance remain open.
## Verification

- Focused `components::overlay::menu::tests` module: 11/11 passed on 2026-10-21.
- Combined overlay tests: 53/53 passed.
- Fresh isolated `./scripts/cargo-headstart check -p rustui --lib --locked --offline`: passed.
- Fresh isolated `./scripts/cargo-headstart check -p rustui-playground --all-targets --locked --offline`: passed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/overlay/menu.rs` and scoped `git diff --check`: passed.

The fresh check logs are `target/headstart/evidence/menu-fallback-fresh-rustui-check-20261020.log` and `target/headstart/evidence/menu-fallback-fresh-playground-check-20261020.log`. Native accessibility-tree and screen-reader acceptance remain open.
