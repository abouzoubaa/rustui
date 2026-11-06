# M831 — Stage 10 P3K run ID collision

The candidate pipeline's default P3K run ID previously used only a UTC
timestamp to the nearest second. Separate candidate pipelines starting in the
same second could select the same run ID; P3K creates its `patch-checks/<id>`
directory exclusively and would reject the colliding run. Add a short UUID
suffix so the default remains timestamp-readable and distinct across concurrent
runs.

The focused deterministic regression fixes the clock and supplies two distinct
UUIDs, proving two IDs from the same second differ and satisfy the safe ID
shape. This prevents default run directory collisions; it does not establish
candidate acceptance, build or correctness, GUI or performance, review,
approval, or promotion.
