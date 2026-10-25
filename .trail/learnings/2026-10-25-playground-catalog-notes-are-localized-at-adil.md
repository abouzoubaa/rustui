---
title: Playground catalog notes are localized at runtime
author: Adil
date: 2026-10-25
branch: codex/production-consolidation-01a172cd
touches:
  - apps/rustui-playground/src/catalog.rs
  - apps/rustui-playground/src/lib.rs
  - apps/rustui-playground/locales/en/lab.ftl
---

The visible component description is LocalizedText(item.note_message_id), not CatalogItem.note. Updating a static CATALOG note alone leaves English/French/Arabic runtime descriptions stale. Keep lab-catalog-note-<key> aligned with the implemented story and validate resolved catalog text, as the Outline catalog regression now does.
