# M687 — D1c GeometryReader preview specimen

## Contract

The canonical compiled-preview gallery exposes the public `GeometryReader` API
through a local factory. The serializable descriptor contains only bounded
`width` and `height` numbers (80–320 and 64–240 logical pixels). The factory
uses those dimensions for the reader and renders the measured native size as
text. The reader identity is scoped to the preview generation. No closure,
coordinate observation, owner mutation, or runtime state is serialized.

## Validation

The focused regression checks schema bounds, default dimensions, and descriptor
import/export round-tripping for edited dimensions. This source-level check does
not establish native layout rendering, resize behavior, accessibility, or
cross-platform acceptance.
