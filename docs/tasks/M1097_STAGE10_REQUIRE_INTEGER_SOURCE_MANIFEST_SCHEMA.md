# M1097 — Stage 10: require an integer source-manifest schema version

Candidate evidence aggregation now requires `source-hash-manifest.json` to
contain the exact integer schema version `1`. JSON boolean `true` is no longer
accepted through Python's `bool`/`int` equivalence.

The focused regression reproduced the invalid acceptance before the fix and
passed afterward; Python bytecode compilation passed. No candidate was run.
