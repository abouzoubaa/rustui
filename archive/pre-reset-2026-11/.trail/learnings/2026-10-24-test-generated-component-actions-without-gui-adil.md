---
title: Test generated component actions without GUI
author: Adil
date: 2026-10-24
branch: work/m118-diagnostics-provenance
touches:
  - crates/rustui-core/src/actions.rs
---

For a Playground component using self.action(handler), a source regression can exercise the generated component Action by calling .dispatch(&mut owner), which tests the handler mutation without constructing GPUI App/Window. This does not exercise RenderContext control binding or the rendered Menu click path; keep visual, keyboard, and accessibility acceptance separate.
