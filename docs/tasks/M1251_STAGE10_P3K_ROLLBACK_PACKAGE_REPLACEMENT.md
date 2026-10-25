# M1251 — Stage 10 P3K rollback package replacement race

P3K rollback now retains the device/inode identity captured from the opened
disposable package directory and passes it to descriptor-relative cleanup.
Recursive cleanup verifies each opened directory against the identity observed
before descent and verifies non-directory entries before unlinking them. This
prevents rollback from descending into a replacement package installed at the
same output path after the original package descriptor was opened.

The focused regression replaces the package directory as rollback begins. It
confirms rollback is reported as failed while both the replacement contents
and the detached original package remain present.

Focused check passed 1/1:

```sh
python3 scripts/tests/test_apply_candidate_patch.py -k rollback_preserves_replaced_package_directory
```

This closes one local rollback cleanup race only. Candidate execution,
current-source GUI and performance evidence, independent review, approval, and
promotion remain open.
