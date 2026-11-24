# G5 owner session

Run from the checkout for draft PR #177, on the Mac and display used for the latency measurements. Keep the apps windowed, use the same power source for both arms, and grant the terminal Screen Recording access before the physical-key captures.

```sh
./scripts/g5_owner_session.sh
```

The script builds once before collecting, prints each action when needed, and saves into a timestamped folder under `target/headstart/performance/`. It runs sync on before sync off. To repeat one part, run `./scripts/g5_owner_session.sh visual`, `./scripts/g5_owner_session.sh power`, or `./scripts/g5_owner_session.sh physical`. An optional second argument selects a new output folder.

| Part | What to do | Saved under each sync arm |
| --- | --- | --- |
| Playground tearing check | Type and backspace quickly in the F0 editor, fling the F0 Collection up and down, drag a node in NodeEditor, then switch between Light and Dark appearance in the Card gallery. Look for horizontal seams, split glyphs, offset parts of one moving object, or different themes on two parts of one frame. Quit the Playground with Command-Q and enter what you saw. | `tearing.txt`, `playground.log` |
| Power | Authorize the script's `sudo powermetrics` prompt and leave the machine alone while it runs typing, hover, drag and idle for 60 seconds each after warmup. | `power/summary.json` and raw sampler and frame logs per scene |
| Physical typing | Press Return in the terminal when prompted. With the Lab editor focused, wait for “Start typing now,” then alternate X and Delete (Backspace) for 60 seconds without modifiers or holding a key. Match the pace for the second arm. The Lab closes after each capture. | `physical/summary.json`, raw events, screen observations, load snapshots and run configuration |

The physical arm timestamps hardware key events using `NSEvent.timestamp`. Compare sync on/off using the ScreenCaptureKit display timestamp of the visible version marker. Metal `presentedTime` is recorded for both arms. It is not comparable across the sync modes. The Playground visual check and privileged power collection are owner-run.
