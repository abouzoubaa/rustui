# M845 — Stage 10: verify the copied Dialog candidate patch

The Dialog candidate preparer checked the approved patch's hash before doing repository-wide inventory and snapshot preparation, then copied it without checking the copy. A patch replacement during that interval could leave `approved.patch` containing bytes that do not match the manifest's approved `patch_sha256`, while those bytes were used to construct the candidate snapshot.

The preparer now rechecks the source immediately before copying and verifies the copied file against the approved hash. The focused regression simulates the copied artifact being replaced and confirms preparation rejects it. `python3 -m unittest scripts.tests.test_prepare_dialog_focus_rebase_candidate` passed (1 test).

This closes only this preparation-copy integrity gap. Candidate source correctness, build, native GUI and performance acceptance, review, approval, and promotion remain open.
