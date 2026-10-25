# M1244 — Stage 10 / reject M274 executable suffix aliases

P3M follow-up validation now requires the normalized final component of each M274 command executable to equal `cargo-headstart`. A rehashed event naming `fake-cargo-headstart` is rejected instead of contributing a passed build gate.

Focused regressions covered the suffix alias, the wrong build/check step, and valid linked M274/M276 reconciliation (3/3). Python syntax compilation and scoped whitespace checks passed.

This validates local evidence-record consistency. It does not prove the command ran, accept the candidate, establish GUI or performance results, approve a patch, or promote it.
