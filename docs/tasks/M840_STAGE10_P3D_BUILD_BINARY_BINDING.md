# M840 — Stage 10 P3D build and binary provenance binding

The P3D visual-evidence validator checked that the build-record and plan files
matched their declared hashes, but it did not require the plan to reference
that exact build record. It also trusted matching binary hashes copied into
the records without checking the binary file itself. A structurally valid
visual record could therefore combine unrelated build evidence or retain a
stale binary hash after the executable changed.

Validation now requires the plan's build-record reference to equal the visual
record's reference, requires the plan and build record to agree with the
declared binary path and hash, and verifies the referenced binary's bytes.
Focused regressions cover a plan referencing another build record and binary
drift after the build and plan records were written.

`python3 scripts/tests/test_validate_p3d_visual_evidence.py` passed (7 tests).
This closes record-level provenance consistency only; it does not establish
candidate GUI acceptance, pixel equivalence, native input/accessibility, or
performance acceptance.
