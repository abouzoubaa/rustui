# RUSTSEC-2026-0382 dependency repair

RustUI updates the locked registry package `rustls-webpki` from 0.103.15 to
0.103.16, the first fixed release in its 0.103 series. The advisory concerns
distrusted certificate authorities. GPUI 0.3.9 and Components/Base 0.7.3 stay
unchanged.

Sources:

- [RustSec advisory](https://github.com/RustSec/advisory-db/blob/main/crates/rustls-webpki/RUSTSEC-2026-0382.md)
- [Official crate archive](https://static.crates.io/crates/rustls-webpki/rustls-webpki-0.103.16.crate)
- Archive SHA-256: `4443b1b6d39d7c663139942cc4558262562d2c84123bb6282ef177009ca8b281`.
- The archive checksum matches the crates.io API and sparse index.

The vendor directory contains the unmodified published crate and its generated
Cargo checksum inventory. Its ISC license text is unchanged (SHA-256
`5b698ca13897be3afdb7174256fa1574f8c6892b8bea1a66dd6469d3fe27885a`).
The lock change contains only this package's version and checksum. Its dependency
list, features and declared Rust 1.71 minimum remain unchanged. Normalizing that
version yields the same 2,594 resolved dependency edges as the previous lock.

Current evidence: locked offline metadata resolves 845 packages and seven
workspace members. The all-target reverse graph reaches this package through
`rustls` 0.23.46. Explicit macOS and wasm32 graphs do not select it. These are
dependency-resolution checks, not runtime TLS tests or support claims for other
platforms. Raw verification is preserved outside the checkout in
`/tmp/rustui-production-webpki-verification.log` and the detached candidate's
evidence directory.

To restore this dependency, use the official archive above, verify its checksum,
extract it without source edits, and generate `.cargo-checksum.json` from the
archive's package checksum and individual file hashes. Verify resolution with
`./scripts/cargo-headstart metadata --locked --offline --format-version 1` and
the corresponding `tree --locked --offline --all-targets -i
rustls-webpki@0.103.16` command. Avoid unconstrained lock updates: a trial update
also changed an unrelated `tempfile`/`getrandom` edge, which this repair excludes.
