# M1228 — Recheck P3 candidate inventory file identity

`PinnedDir.inventory()` now verifies the opened file is regular, has one hard
link, and matches the device/inode observed through the no-follow directory
entry. After hashing, it rechecks the descriptor and path identity. This
rejects a file replaced with an external hard link between the initial stat
and open.

The focused regression injects that replacement race and confirms inventory
rejects it.

```sh
python -m unittest \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_inventory_rejects_hard_link_replacement_between_stat_and_open
```

Result: passed (1/1). Candidate execution, GUI/performance acceptance, review,
approval, and promotion remain separate gates.
