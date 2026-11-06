# Menu trigger ID validation

`MenuView::id` now rejects empty or whitespace-only identities at the builder
boundary. This preserves the O1 contract that each dropdown trigger has an
explicit stable ID before lowering.

The focused regression passed with the locked offline RustUI library test
filter; see `target/headstart/evidence/menu-trigger-id-validation-20261020.log`.
This is source-level validation only. Native menu identity and complete
accessibility acceptance remain governed by `docs/tasks/O1_MENU_DROPDOWN.md`.
