# P3H Cargo manifest hash drift

P3H now hashes every candidate-local `Cargo.toml` under the selected workspace
before and after `cargo metadata`. A manifest change during metadata inspection
invalidates the report, just as Cargo.lock and the selected Cargo config chain
already do. A mocked metadata subprocess that changes a path package manifest
reproduced the gap: P3H previously returned `valid` while reporting the old
package identity. The focused regression now rejects that case.

This check covers Cargo manifest input consistency only. It does not replace
P3G's full source snapshot hashes or establish candidate acceptance.

The hash walk is scoped to the selected workspace because `validate_metadata`
rejects path packages whose manifests are outside that workspace, and rejects
registry packages whose manifests are outside `workspace/vendor/`. Thus Cargo
may inspect a candidate-local sibling path package before validation, but that
metadata cannot produce a valid P3H report.
