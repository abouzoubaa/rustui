# P3 paired native visual comparison

## M283 — Stage 10 / P3

The M272 baseline and candidate were copied into disposable source trees, then
given the same two overlays: the M277 overlapping/clipped Quad specimen and a
direct Canvas route that bypasses the legacy nested Basics gallery. Both copies
built independently in locked/offline mode. Their separately launched
Playground windows rendered the gold-first translucent-blue overlap and clipped
the overhanging Quads at opposite Canvas edges. Native accessibility trees
exposed the specimen label and Canvas container on both sides. The observed
output was visually equivalent for this bounded fixture.

The paired visual gate for this bounded fixture is now passed: M287 retained
identical baseline and candidate window screenshots and recorded their hashes.
The source overlays mean these builds are disposable comparison copies, not
the unchanged M272 snapshots. Performance acceptance remains unsupported by
M276's ordered 131,071-Quad regression, thresholds remain undefined, and
candidate promotion remains open. This GUI comparison does not change the M272
intake result or candidate identity.

Evidence is under
`target/headstart/candidates/m283-p3-gui/evidence/`. Build outputs live in the
paired disposable `baseline/` and `candidate/` trees. The overlay patch files
and binary hashes are recorded in
`m283-paired-visual-comparison-20261020.json`.

## M287 — Stage 10 / P3 screenshot evidence follow-up

The baseline and candidate were relaunched sequentially from the existing M283
disposable builds using the same bundle ID (`dev.rustui.playground.validation`)
and the same direct Quad Order fixture. CUA inspection confirmed the fixture
and its native label/container on both sides. Targeted window captures were
retained at `target/headstart/candidates/m283-p3-gui/evidence/m287-baseline.png`
and `m287-candidate.png`; each is 2732 × 1732 pixels. Their SHA-256 hashes
are identical (`55f8cdc3dc33a224ac2b11d96cdd54981fbfe95c6ea0bf89468aaf6d8515ad9f`),
so the captured pixels match exactly. This closes M283's retained-screenshot
gap for the bounded GUI comparison. The two-Quad scene uses V2's small-input
stock-sort fallback and does not exercise its changed counting-sort path. M276
performance acceptance and candidate promotion remain open. The prepared M272
snapshots, candidate patch, vendored source, and canonical Playground were
untouched.
