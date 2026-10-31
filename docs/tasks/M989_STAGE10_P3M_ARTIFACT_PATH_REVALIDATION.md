# M989 — Stage 10 / revalidate artifact paths after P3M descriptor reads

P3M artifact reads now verify the opened descriptor's identity/version and
reopen the leaf through its pinned parent directory after reading, rejecting
replacement of the referenced artifact path.

The deterministic M287 screenshot replacement regression and normal M287
aggregation test passed (2/2).

No candidate was run. Review, approval, and promotion remain open.
