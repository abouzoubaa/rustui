# G5 owner session

Run on the measurement Mac from the G5 closeout checkout. Keep the same
windowed display setup and power source for all three arms. Give the terminal
Screen Recording permission before the physical-key captures.

```sh
./scripts/g5_owner_session.sh
```

The script builds before collecting, prints the action for each arm, and
saves records below `target/headstart/performance/`. An optional second
argument selects a new output directory.

| Arm | Profile | Windowed Metal display sync |
| --- | --- | --- |
| `profile-off` | Off | On |
| `profile-on` | On | On |
| `profile-on-sync-off` | On | Off, owner comparison only |

| Part | What to do | Saved under each arm |
| --- | --- | --- |
| Visual | Type and backspace rapidly in the F0 editor. Fling the F0 Collection down and back up. Drag a node in NodeEditor. Switch between Light and Dark in the Card gallery. Look for horizontal seams, split glyphs, offset parts of a moving element, and different themes across the same frame. Quit the Playground with Command-Q and enter what you observed. | `tearing.txt`, `playground.log` |
| Power | Authorize `sudo powermetrics` when prompted, then leave the machine alone. The script samples typing, hover, drag and idle for 60 seconds each after warmup. | `power/summary.json`, raw power and frame logs |
| Physical typing | Press Return to start each arm. Wait for “Start typing now,” then alternate physical X and Delete (Backspace) without modifiers for 60 seconds. Use the same pace in all three arms. The Lab closes automatically. | `physical/summary.json`, native events, screen observations and load snapshots |

You can repeat an individual part with the `visual`, `power`, or `physical`
argument. Only `all` and `power` ask for sudo. Each arm saves its profile and
sync settings in `mode.txt`. The automated checks also read the Metal layer
setting from the recorded frames.

Physical input uses the key event's `NSEvent.timestamp`. Compare all three
arms on the ScreenCaptureKit `displayTime` clock. The saved Metal
`presentedTime` values use mode-specific semantics. The visual check covers
physical tearing that a compositor capture may miss. Display sync off
remains pending the owner's session.
