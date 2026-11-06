# M1031 — Follow Cargo resolve edges in dependency-boundary checks

The dependency-boundary verifier previously looked for resolved package IDs in
`packages[].dependencies`, but Cargo stores resolved graph edges in
`resolve.nodes[].deps`. The verifier therefore missed reachable transitive
declarations and local path dependencies.

The verifier now follows Cargo resolve nodes while preserving direct
declaration checks for `--no-deps` metadata. A regression covers a transitive
optional Allui declaration. The focused test module passed:

```text
python3 -m unittest scripts.tests.test_verify_dependency_boundary -v
Ran 5 tests
```

No candidate run or broader Stage 10 acceptance/review/promotion was performed.
