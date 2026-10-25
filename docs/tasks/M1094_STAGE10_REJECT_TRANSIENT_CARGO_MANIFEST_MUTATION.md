# M1094 — Stage 10: reject transient Cargo manifest mutations

P3H source reporting now compares Cargo manifest content hashes and file and
directory versions across Cargo metadata inspection. Including inode and ctime
detects a manifest changed and restored while metadata was being collected.

The focused mutate-and-restore regression passed 1/1. No candidate was run;
independent build/correctness and GUI/performance evidence, review, approval,
and promotion remain open.
