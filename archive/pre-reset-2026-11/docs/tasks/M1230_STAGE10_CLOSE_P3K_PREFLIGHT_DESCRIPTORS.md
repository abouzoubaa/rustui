# M1230 — Close P3K package descriptors after preflight rejection

Pinned P3K preflight now closes both package-directory descriptors when
manifest, tree, hash, or patch validation fails. A focused regression forces a
baseline inventory rejection and confirms both descriptors are closed. It
failed before the fix and passed afterward (1/1).

This closes a descriptor leak in the patch applier. Candidate execution,
downstream build/correctness, GUI/performance evidence, independent review,
approval, and promotion remain separate gates.
