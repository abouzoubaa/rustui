---
title: Verify composed Web patch images after RTL wrapping
author: Adil
date: 2026-10-26
branch: codex/production-consolidation-01a172cd
touches:
  - scripts/tests/test_gpui_web_semantic_adapter.py
---

The approved RTL wrapped-row patch changes the Web text_system.rs POST image expected by the older semantic adapter verifier. Verify the RTL POST exactly, reverse it in a temporary image, and prove the semantic stack against that intermediate image. Semantic lifecycle fixtures must copy the same intermediate image; never rewrite the selected vendor checkout for this proof.
