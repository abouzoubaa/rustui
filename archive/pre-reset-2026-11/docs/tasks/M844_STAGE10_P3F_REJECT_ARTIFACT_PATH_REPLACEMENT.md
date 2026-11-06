# M844 — Stage 10 P3F reject artifact path replacement during validation

The P3F record validator now checks the artifact descriptor remained stable while hashing and reopens the no-symlink path to ensure it still names the hashed inode. A focused regression replaces an evidence artifact at its path during the read; the validator rejects the record. This is local record-integrity hardening only. It does not establish candidate build, correctness, GUI, performance, approval, or promotion acceptance.

Validation: `python3 -m unittest scripts.tests.test_validate_candidate_evidence`.
