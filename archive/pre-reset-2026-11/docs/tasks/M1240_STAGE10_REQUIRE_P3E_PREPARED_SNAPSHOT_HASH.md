# M1240 — Require the P3E prepared candidate snapshot hash

P3E intake validation now requires a valid lowercase SHA-256 for the prepared
candidate snapshot before comparing the patch-plan binding. Previously, if
both the prepared snapshot and patch plan omitted the hash, `None == None`
passed validation. The focused regression removes both hashes while refreshing
the enclosing records and confirms rejection; it passed (1/1).

This validates intake bindings only. The candidate pipeline was not run;
candidate correctness, GUI/performance evidence, independent review, approval,
and promotion remain separate gates.
