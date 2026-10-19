# Themes

`theme.schema.json` defines RustUI appearance format version 1. A runtime pack
combines a platform snapshot with component records and semantic role references.
`macos27.json` is currently the only runtime pack. Other platform JSON files are
planning records and do not imply runtime support.

## What runtime activation applies

RustUI validates the platform snapshot, selects its light or dark palette, and
maps every color role exposed by the pinned GPUI Components theme. It also sets
the shared system font, body size, and general radii. RustUI `Text` reads that
active theme; `Card` reads profile spacing and radius; collection/table/list
selection uses the mapped selection color.

The `components` object also preserves imported design records. At present,
`button` is the only record with a typed RustUI resolver. Other records are not
automatically applied just because they are present in JSON; they may include
reference artwork details that have no GPUI runtime equivalent. Do not report a
component-specific metric as active unless its RustUI rendering path consumes
it.

## Loading and activation

Initialize GPUI Components, load a validated theme, then activate it:

```rust,ignore
gpui_component::init(cx);
rustui::theme::activate_theme(
    &document,
    Some(AppearanceMode::Dark),
    cx,
)?;
```

Theme activation updates the GPUI Components global theme and publishes the
resolved RustUI theme through GPUI application globals. It does not create
native Liquid Glass blur; native composition remains a separate capability.

## Coverage check

Run this after changing the adapter or GPUI Components snapshot:

```sh
python3 scripts/verify_theme_adapter_coverage.py
```

It fails if a color role is added to the pinned `ThemeColor` without an explicit
RustUI mapping.
