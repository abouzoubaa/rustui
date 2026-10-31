# M1112 — Stage 10 P3F classify artifact replacement during hashing

## Contract

P3F must distinguish an artifact gaining a hard link from an artifact whose
identity or link state changes while its digest is being read.

## Change

An increasing link count still reports a gained hard link. Unlink or
replacement during hashing, including the opened inode's link count dropping
to zero, now reports that the artifact changed.

## Evidence

`python3 -m unittest scripts.tests.test_validate_candidate_evidence` passed
(19 tests), including the replacement-during-digest regression. Scoped
whitespace checks found no errors.

This is focused evidence-validation behavior; no candidate was run or
accepted.
