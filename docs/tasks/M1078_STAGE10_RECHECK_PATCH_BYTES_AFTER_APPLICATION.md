# M1078 — Stage 10 / recheck patch bytes after application

The P3K applier now rereads each patch through the pinned candidate directory
after application and compares it with the approved digest. Previously, the
post-check compared cached in-memory bytes with hashes derived from those same
bytes, so it could miss an on-disk mutation during application. A focused
regression confirmed rejection and schema-3 rollback (1/1). No candidate was
executed or promoted.
