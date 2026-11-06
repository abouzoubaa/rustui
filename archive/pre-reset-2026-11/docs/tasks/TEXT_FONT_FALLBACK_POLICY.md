# Text Font Fallback Policy

## Scope

Allow callers to supply an ordered fallback family list to RustUI `Text` and
`LocalizedText`, using the pinned GPUI `FontFallbacks` API. With no policy,
text lowering keeps its existing behavior.

## Contract

- The caller provides family names and their order; RustUI does not infer them
  from a locale or language.
- Empty lists and whitespace-only family names are rejected.
- The fallback refinement is applied without setting or replacing the inherited
  font family, features, weight, or style.
- GPUI remains responsible for shaping and platform font resolution.

## Playground example

The M42 native review observed the sample with its caller-supplied policy set:
Latin `Hello`, connected Arabic `مرحبًا`, Chinese `你好`, and French
`LocalizedText` text `Fermer` were visible. The bounded record is
`target/headstart/milestone42-text-fallback-gui.json`. The suite reported 119
passing tests, including rejection of empty/blank names and preservation of the
caller's family-name order.

## Limits

This does not translate text, set locale or bidi behavior, or guarantee that a
family exists on the platform. The rendered sample does not establish actual
operating-system font selection or glyph provenance. Native fallback samples
in other locales remain pending.
