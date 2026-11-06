# P2a: Build provenance capture

## Purpose

`scripts/record_build_provenance.py` writes a JSON record that associates an
existing build log with workspace metadata captured at the time the script runs.
It does not run Cargo or rustc, interpret a log as proof of success, validate
dependency freshness, or measure runtime performance.

## Usage

From the RustUI workspace, after a build log already exists:

```sh
python3 scripts/record_build_provenance.py path/to/build.log
```

By default, it creates a unique file in `target/headstart/evidence/` and refuses
to overwrite an existing file. An explicit destination can be supplied:

```sh
python3 scripts/record_build_provenance.py path/to/build.log \
  --output target/headstart/evidence/my-build.json \
  --command './scripts/cargo-headstart check --workspace --locked' \
  --profile dev --exit-status 0 \
  --cargo-version 'caller supplied Cargo version' \
  --rustc-version 'caller supplied rustc version'
```

The command, profile, exit status, and version strings are optional caller
declarations. An exit status of zero is recorded as `observed_success` only as
caller-supplied metadata; absent an exit status, status remains `unverified`.
The log is hashed but not parsed for success. Dependency reuse is always
`not_evaluated`; use `scripts/verify_dependency_reuse.py` separately when its
Cargo JSON input and narrow freshness assertion are appropriate.

## Recorded fields and limits

- SHA-256 and size for the supplied log, workspace manifest, lockfile,
  `rust-toolchain.toml`, `scripts/cargo-headstart`, and `.cargo/config.toml`.
- SHA-256 and size for regular files under `crates/` and `apps/`, excluding
  `vendor/`, `target/`, `.git/`, and `node_modules/` trees.
- Current platform, current Git commit when `git rev-parse HEAD` is available
  (otherwise `null`), and a fixed allowlist of build-related environment
  settings. No arbitrary environment variables are copied.
- Caller-declared command/profile/exit status and optional version strings.
  The utility does not query Cargo or rustc, and does not infer historical
  compiler flags from the launcher or current environment.

Hashes, platform, commit, and environment are captured when the utility runs.
They may differ from the build-time state that produced the supplied log. The
record has separate `captured_current_workspace` and
`build_facts_declared_by_caller` sections to make that distinction explicit.
Missing or unreadable source inputs are represented with a status in the JSON;
an absent or unreadable build log is an error and no record is written.

This utility is provenance bookkeeping only. It adds no dependency, changes no
launcher behavior, and is not an optimization benchmark or upstream promotion
workflow.

## Update 2026-10-19: Cargo feature selection

The recorder accepts `--features` and stores its value under
`build_facts_declared_by_caller.features`. This records the feature selection
reported by the caller; it does not recover or verify historical Cargo flags.
The focused regression test is
`scripts/tests/test_record_build_provenance.py::test_records_caller_declared_features_and_profile`.
