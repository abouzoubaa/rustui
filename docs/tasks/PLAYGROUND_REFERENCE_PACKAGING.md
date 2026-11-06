# Playground reference packaging

The canonical Playground's native app bundle stores its SVG reference catalog
under `Contents/Resources/references`. Runtime lookup checks that bundle
resource directory first and falls back to
`apps/rustui-playground/references` for source-checkout runs.

`package-app.sh` copies the complete reference tree. The current source tree is
about 727 MB. The packaging smoke test supplies a small nested fixture through
`RUSTUI_PLAYGROUND_REFERENCES_DIR`, so validation checks the real packager
without duplicating the full catalog. No GUI or packaged-app launch acceptance
is claimed by these tests.

Focused evidence:

- `target/headstart/evidence/m517-playground-reference-resolution-tests-20261021.log`
- `target/headstart/evidence/m517-playground-packager-tests-20261021.log`

## M798 failed-promotion rollback

The packager now moves the installed references aside only after staging the new tree, restores the previous directory if promotion fails, and removes staging and backup directories on exit. The focused fixture injects failure into the staging-directory `mv`, then confirms the installed SVG remains, the failed SVG was not published, and no staging directory remains. `scripts/tests/test_package_playground_app.sh` passed. This is packaging failure-path evidence, not GUI or packaged-app launch acceptance.

## M521 stale-reference replacement

Each packaging run stages a fresh SVG catalog and replaces the bundle's prior references directory, so removed source assets do not remain in the app. The packaging test runs twice with different fixture trees and verifies the stale SVG disappears while the replacement is copied. Evidence: `target/headstart/evidence/m521-playground-reference-replacement-20261021.log`.
