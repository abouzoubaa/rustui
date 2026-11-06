# Current-candidate native controls and theme check — 2026-11-05

## Candidate identity

- Worktree: `/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui`
- Branch and source revision: `codex/production-consolidation-01a172cd`, `d8d3314e8269eafd6e70cd6618b0967333635942`.
- Platform: macOS 27.0, arm64.
- Build command: `./scripts/cargo-headstart build -p rustui-playground --locked --offline`.
- Build result: succeeded; Cargo reused the existing `dev` artifact and finished in 1.13 seconds. Log: `target/headstart/evidence/native-mvp-current-candidate-build-20261105.log`, SHA-256 `65d93331307510eb605d9043613e63e78ba058da279af785c92cf89b73369281`.
- Executable: `target/headstart/debug/rustui-playground`, SHA-256 `0aa58c36d9697906392012a894e8b587563d6c104f2996e690a2a61403f4ef5c`. The packaged validation app had the same hash and bundle ID `dev.rustui.playground`.
- Current committed revision differs from the source revision of the reused executable only by documentation. Three pre-existing popup-related working-tree changes remained; their combined diff hash was `bd7243857d18cf4194b7b5ad4bf09efd5c6400ff2bde36c09e764b61863b2939`. The two vendor files matched the approved v4 patch post-images. No source or popup files were edited in this check; the proposed v5 patch was not applied.

## Observed results

- In the Text inputs story, entering `MVP candidate` updated the owner value and its bound Name, Search, and Notes fields.
- Turning on the Enabled switch and selecting Accepted updated their displayed state. Returning Accepted to off and clicking the disabled Accepted option left it off.
- Tab moved from Accepted to the Radio control, skipping the disabled checkbox. Space selected Radio; Shift-Tab returned to Radio from the next checkbox. Home then Right changed the Slider from 35 to 1.
- Keyboard activation changed the visible Stepper quantity from 2 to 3; Tab then Space decremented it back to 2.
- Selecting Kotlin in the Picker changed the owner label to Kotlin and updated the other control sharing that owner value.
- In the Button story, pointer activation changed Count from 0 to 1; Space changed it from 1 to 2. Disabling the story disabled Increment and Reset; clicking disabled Increment left Count at 2 and added no action event.
- Switching Light and Dark changed the visible theme while the Button story remained selected and Count stayed at 2. This check did not assess animation timing or appearance.

## Still open

- After disabling Increment while it had keyboard focus, the focus ring appeared to remain on that disabled button after Tab. The disabling action was issued through the UI automation accessibility action, so this observation alone cannot distinguish a RustUI focus defect from an automation focus artifact. A source-backed, ordinary-input reproduction is still needed.
- This was a bounded combined session, not the complete Basic controls workflow. Full text editing with ordinary keyboard cursor/selection operations and the remaining documented control/group policies were not repeated here.
- Theme switching was visible, but the required ordinary-motion visual path was not observed.
- Native popup placement, outside-click dismissal, parent-window teardown, and owner-state updates were not tested.

The disposable validation process exited and its app bundle was moved to Trash. The build caches and evidence were preserved. The evidence total remains **23/34 (67.65%)** and the acceptance total remains **2/11 (18.18%)**; no acceptance group was closed by this session.
