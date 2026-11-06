# M1215 — Stage 3 record hover exit when hiding the region

The Hover Gallery now counts hiding a currently hovered region as one exit.
The owner clears its hover state before the child unmounts, so a later exit
callback cannot record that transition. Hiding an already unhovered region
does not add another exit.

Verification command:

```sh
CARGO_PROFILE_TEST_DEBUG=0 ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib hiding_a_hovered_region_records_its_exit_once
```

Result: 1 passed, 0 failed; 184 filtered. The regression now calls `exit()`
after hiding and asserts that the delayed callback does not increment the
counter again. The default test profile reused cached dependency metadata
built against a different `std`; rustc reported `E0463` for the dependencies.
The profile override rebuilt them under the current Headstart compiler in the
existing `target/headstart` directory without clearing the shared cache. This
is state-test evidence only; no GUI acceptance was run.

## Native/browser GUI acceptance run sheet

Run only when macOS is unlocked and the shared Playground slot is available.
Identify the owner of any running Playground process; leave unowned processes
untouched. Use the canonical Playground and its existing disposable
validation-app bundle and identifier. Confirm the build contains the current
`apps/rustui-playground/src/stories/hover_gallery.rs` source.

1. Open the Hover Gallery with the pointer outside the region. Record the
   initial status: `Hovered: false · Enters: 0 · Exits: 0 · Child clicks: 0`.
2. Move the pointer into the region. Pass when the status shows
   `Hovered: true`, Enters `1`, Exits `0`.
3. Move to **Hide while hovered** without leaving the region and click it.
   Pass when the region disappears, Hovered is `false`, and Exits becomes `1`.
   Wait for the unmount and any delayed pointer-exit event; Exits must remain
   `1`.
4. Activate **Restore region**. Pass when the region returns, Hovered is
   `false`, and the counters remain Enters `1`, Exits `1`.
5. Keep the pointer outside, focus **Hide while hovered** by keyboard, and
   activate it with Space or Enter. Record the exact Tab sequence and key.
   Pass when the region disappears and Exits remains `1`. If the button cannot
   be focused while the pointer stays outside, mark this step not run and
   record why.

For each step record pass, fail, or not run and the observed status. Capture
the surface used, app and bundle identity, source revision, binary/build
identity, run timestamp, and screenshots or a short recording of the initial,
hovered, and hidden states. Keep native/browser acceptance open until these
observations are recorded. Repeated hide after unmount is covered by the
focused state regression because the Hide button is no longer present.

After the GUI run, quit only the test app launched for this run, remove its
disposable app bundle and standalone test executable, preserve source,
evidence, and shared Cargo caches, and verify that no test app remains running.
