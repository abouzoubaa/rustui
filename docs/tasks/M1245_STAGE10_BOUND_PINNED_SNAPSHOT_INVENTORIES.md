# M1245 — Stage 10 / bound pinned snapshot inventories

The descriptor-pinned intake path now applies P3G's existing 100,000-entry and 4 GiB limits while inventory hashing runs. The limits cover pre-copy source inventory, post-copy source revalidation, and destination inventory, so hashing cannot bypass the limits enforced during copying by M1243.

The focused regression exercises both entry and byte limits (1 test). Python syntax compilation and scoped whitespace checks passed.

This closes a local resource-boundary gap. It does not establish a full candidate-pipeline run or candidate acceptance.
