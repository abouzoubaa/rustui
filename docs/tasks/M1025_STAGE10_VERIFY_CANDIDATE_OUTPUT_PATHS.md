# M1025 — Verify candidate patch output paths after pinned-FD operations

The candidate patch applier operates on output directories through pinned file
descriptors. It previously did not verify that the reported paths still named
those directories before returning `status: passed`. Swapping an output path
after it was opened could make the reported package path resolve elsewhere or
nowhere.

`scripts/apply_candidate_patch.py` now checks the patch-check root, run output,
and package output paths against their pinned descriptors during final
integrity verification.

The focused regression failed before the change and passed afterward:

```text
python3 -m unittest scripts.tests.test_apply_candidate_patch.CandidatePatchApplicationTests.test_rejects_patch_check_root_swap_after_open -v
Ran 1 test, OK
```

No broader tests, build, candidate run, review, approval, or promotion were
performed as part of this milestone.
