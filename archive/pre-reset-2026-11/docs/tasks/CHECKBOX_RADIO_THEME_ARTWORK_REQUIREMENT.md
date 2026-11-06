# Checkbox and Radio theme artwork requirement

`Checkbox` and `Radio` derive their indicator dimensions from the resolved
`vector_path` theme artwork. The shared renderer directly indexes that field
and expects SVG path data plus an identity transform. Missing artwork previously
passed theme validation and could panic during rendering. Theme validation now
requires `vector_path` on every Checkbox and Radio style rule, alongside the
already validated color roles and Radio geometry.

The focused regression removes `vector_path` from each component's body rule
and verifies that validation rejects both malformed themes. This checks missing
artwork only; SVG path-shape and transform validation remain part of the
renderer contract and broader native visual theme acceptance is separate.
