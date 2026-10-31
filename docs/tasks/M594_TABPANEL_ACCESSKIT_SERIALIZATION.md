# M594 — Stage 8 selected TabPanel AccessKit serialization

The production selected-panel builder is factored into `selected_tab_panel_element` and is now tested through GPUI's `Element::write_a11y_info` hook. The regression checks the emitted `Role::TabPanel`, the selected tab's accessible-name override on the AccessKit node, and that the element has an identity. It tests the same builder used by `TabViewControl::render`; it does not render a full native accessibility tree.

Validation: locked/offline focused `rustui` library test passed 1/1 (675 filtered), and the complete TabView test group passed 18/18 (658 filtered), the RustUI library suite passed 676/676, and the locked/offline RustUI all-targets check passed; `rustfmt --edition 2024` passed. Evidence: `target/headstart/evidence/m584-tabpanel-accesskit-tests-20261021.log`, `target/headstart/evidence/m594-tabview-tests-20261021.log`, `target/headstart/evidence/m594-rustui-library-tests-20261021.log`, and `target/headstart/evidence/m594-rustui-all-targets-check-20261021.log`.

Native tab-to-panel association, screen-reader traversal and announcements, focus behavior, RTL, and cross-platform acceptance remain open. The pinned TabBar API exposes no tab-to-panel association setter. This milestone does not alter public API or vendored code.
