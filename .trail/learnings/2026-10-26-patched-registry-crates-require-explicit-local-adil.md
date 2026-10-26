---
title: Patched registry crates require explicit local package selection
author: Adil
date: 2026-10-26
branch: codex/production-consolidation-01a172cd
touches:
  - Cargo.toml
---

Cargo treats registry packages as immutable: changing an approved vendored source image may still reuse the original cached artifact. Select the same pinned patched package with a root [patch.crates-io] local vendor path and matching source-free lock entry. Verify Cargo metadata reports source=null and the intended manifest path, the build log actually compiles that crate, and the bundle executable hash changes. The native macOS focus forwarder verifier enforces this selection while preserving upstream checksum/license/VCS evidence. Do not claim patched code was compiled from a quick cached build alone.
