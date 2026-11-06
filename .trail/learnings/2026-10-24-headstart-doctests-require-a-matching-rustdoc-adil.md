---
title: Headstart doctests require a matching rustdoc
author: Adil
date: 2026-10-24
branch: work/m118-diagnostics-provenance
touches:
  - scripts/cargo-headstart
---

On this host, scripts/cargo-headstart selected the Headstart rustc but stage1/bin/rustdoc was absent, so it fell back to the installed Rust 1.97.1 rustdoc. rustui-macros unit and integration tests passed, then doctests failed with E0514 because cached proc-macro dependencies were built by Headstart. Do not cargo clean to address this; provision a matching stage1 rustdoc or report doctests as unavailable while running --lib --tests separately.
