# T1A: Explicit theme file reload

## Existing-host invalidation

Theme activation writes RustUI's `ActiveTheme` GPUI global after applying the GPUI Components profile. Existing `ComponentHost` instances must observe that global and notify GPUI so RustUI renderers reread their resolved theme on the next frame. `ComponentHost` now holds that observer subscription alongside localization invalidation. The behavior fix is source-level; native appearance-switch rendering remains an acceptance check and is not claimed by compilation alone.

## Scope

`ThemeFile` provides validated file loading and explicit, transactional reloads
through the existing version 1 loader, Button resolver, GPUI Components adapter,
and GPUI application state. It does not start a background thread or watch the
filesystem. A caller may poll with `reload_if_changed` or request `reload` from
an existing GPUI callback.

## API

- `ThemeFile::open(path)` reads and validates a file without changing the app.
- `theme()` returns the opened or most recently activated valid document.
- `reload(mode, cx)` reads, validates, and force-applies the file.
- `reload_if_changed(mode, cx)` applies only when file content or requested
  appearance mode differs from the last successful activation. It compares
  content, not timestamps or file size.
- Both reload calls return `true` when activation succeeds and `false` when
  `reload_if_changed` finds no change. The first reload always activates.

Read, parse, resolve, and apply errors leave the stored valid document and
activation fingerprint unchanged, allowing a repaired file to be retried.
Calling `reload` forces activation if another caller changed GPUI's global
theme without changing this file.

## Verification boundary

Focused unit tests cover content changes, unchanged content, mode changes,
forced activation, read and parse failures, repair, and activation failure using
an injected activation seam. These tests do not claim native GPUI theme
activation or visual acceptance. Workspace compilation and full test
orchestration remain with the root task.
