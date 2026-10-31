---
title: Component state accessor name collisions
author: Adil
date: 2026-10-22
branch: work/m118-diagnostics-provenance
touches:
  - crates/rustui-macros/src/component.rs
---

The #[component] macro emits five inherent methods per #[state] field: getter, setter, updater, binding, and state accessor. Distinct fields can collide (for example, count generates set_count while a state field named set_count generates the same getter), causing a downstream duplicate-definition error. validate_state_accessor_names now rejects cross-field collisions at the later field span with a direct diagnostic before code generation. Regression: cargo-headstart test -p rustui-macros generated_state_accessors_reject_cross_field_name_collisions.
