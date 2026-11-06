# M65 Playground automatic theme data reload

The existing Theme and Layout story has an opt-in **Start auto** control. It
checks the theme source every 350 ms using GPUI's `BackgroundExecutor::timer`,
with each file read, validation, and activation performed on the GPUI app
thread by `ThemeFile::reload_if_changed`. **Stop auto** cancels the stored
native task. The story's `on_unmount` also cancels it; the task captures only a
weak reference to its state. No polling runs until enabled.

The default file remains `themes/macos27.json`. Set
`RUSTUI_THEME_RELOAD_PATH` before launching Playground to point this story at
an isolated copy for development or verification. The variable affects only
this story; it does not replace the application startup theme or the theme
inspector's source. Avoid editing the checked-in theme to verify this feature.

On each tick, the story reads the active RustUI appearance mode from the app
theme state. An outer Playground Light/Dark action therefore remains in
control; a subsequent file edit is applied in that mode. The story's own
Light/Dark buttons still explicitly reload and switch mode. If no active mode
is available, the tick waits without choosing one.

`ThemeFile` compares file content and appearance mode. Unchanged content does
not activate or refresh again. Read, parse, or activation failures keep the
last successfully applied theme and fingerprint, show the error once, and are
retried on later ticks. Repairing the file applies the new valid version. If
the repair restores the exact last-good bytes, the next successful unchanged
check replaces the error with one recovery status update; later unchanged
checks remain quiet. The story refreshes windows only when theme data applies,
the status message changes, or the enable state changes. Polling does not
recompile Rust code or serialize general views/state; it is an app-only
development feature.

## Acceptance procedure

1. Copy `themes/macos27.json` to a temporary path and launch Playground with
   `RUSTUI_THEME_RELOAD_PATH` set to that path.
2. Open Theme and Layout, select **Start auto**, then make a valid same-path
   theme edit. Confirm the visible theme and status update once.
3. Leave the file unchanged across several ticks. Confirm no repeated
   activation or status changes. Use both the story and outer Light/Dark
   controls, then edit again and confirm the active appearance is respected.
4. Save invalid JSON or temporarily remove read access. Confirm an error is
   shown and the last good theme remains. Repair the file and confirm it
   applies.
5. Select **Stop auto**, edit the file, and confirm no automatic update.
   Re-enable, then remove the story from the active layout and confirm its
   task is cancelled.

## M65 executed acceptance

The native review used the isolated file at
`target/headstart/verification/m65-theme-reload.json`; the checked-in canonical
theme was not modified and the verification file was restored after the run.
Start auto activated the current Dark appearance. A valid same-path edit applied
the dark window/content change; invalid JSON showed a reload error while the
last good green appearance remained; restoring the exact last-good bytes cleared
the error. The outer Light control remained authoritative across later file
edits. While disabled, an edit did not apply; re-enabling applied it. Switching
away from the story while enabled and then editing did not activate the later
change, consistent with owner task cancellation.

Evidence: `target/headstart/milestone65-theme-reload-gui.json`; bundle SHA-256
`e917a357fbe391eb428d0cb029d87fecb41cf7aba6e9dc128804618a4dd5d678`.
The M63–M65 workspace suite passed 200 tests, 0 failed across 12 binaries
(`target/headstart/milestone63-65-final-tests.log`). Build succeeded with all
three pinned GPUI dependencies fresh
(`target/headstart/milestone63-65-reuse-report.json`); the locked/offline
all-targets check exited 0
(`target/headstart/milestone63-65-all-targets-check.log`).

Limits: this is 350 ms development polling of theme data, not Rust code hot
reload or a filesystem watcher. No exact activation counter or independent
unchanged-poll quiescence measurement was made. Large-file latency/CPU,
missing-file/permission native cases, and multi-root isolation remain
unverified. Owner release is supported by no later activation plus source task
cancellation, not an OS file-read trace. No performance claim is made.
