# M1090 — Stage 10: derive the source tree digest from the captured inventory

The candidate pipeline now calls `inventory(source_path)` once per snapshot
and derives both the recorded file map and `tree_sha256` from that same map.
This prevents the immutable source-hash manifest from combining file hashes
from one scan with a tree digest from a later scan.

The focused alternating-inventory regression passed 1/1. The candidate
pipeline was not executed against a real candidate. Candidate execution,
independent build/correctness and GUI/performance evidence, review, approval,
and promotion remain open.
