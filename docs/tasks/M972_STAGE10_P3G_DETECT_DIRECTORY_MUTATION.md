# M972 — Stage 10 / detect directory changes during snapshot inventory

## Change

P3G snapshot inventory detects directory metadata changes while enumerating,
including a regular file added after the directory entries were collected.

## Focused evidence

`test_rejects_directory_entry_added_after_enumeration` passed (1/1).

## Remaining gates

No candidate outputs were run. Candidate correctness/build evidence, independent
review, approval, and promotion remain open.
