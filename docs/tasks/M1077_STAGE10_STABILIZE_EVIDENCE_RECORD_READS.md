# M1077 — Stage 10 / stabilize evidence-record reads

The candidate evidence validator now rejects evidence records with external
hard-link aliases, detects a link added during the read, compares descriptor
identity and version metadata before and after reading, and reopens the path to
reject replacement. A deterministic hard-link-during-read regression and the
valid-record case passed (2/2). No candidate was executed or promoted.
