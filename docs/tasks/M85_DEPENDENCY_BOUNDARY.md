# M85 — Standalone dependency boundary guard

`scripts/verify_dependency_boundary.py` uses Cargo metadata from the project Headstart launcher to check that RustUI workspace members do not declare or resolve Allui, including optional dependencies. It also rejects local path dependencies outside the RustUI workspace while allowing checked-in vendored crates.

The checker traverses the resolved dependency graph and inspects dependency declarations for every reachable package, so an inactive optional transitive dependency is still reported. It does not scan Rust source; a public re-export of Allui would require a dependency and is therefore caught by the manifest checks.

Validation on 2026-10-19:

- `python3 -m unittest scripts.tests.test_verify_dependency_boundary -v` — 4 tests passed, including vendored-path allowance, transitive and optional Allui detection, and external local-path detection.
- `python3 scripts/verify_dependency_boundary.py` — passed against the current locked workspace.

This protects the standalone dependency boundary; it does not diagnose or change theme behavior.
