# GPUI Fast native acceptance

Run on the macOS Playground built from the f0-integration branch with GPUI Fast.
1. Open the 5,000-row collection story and scroll rapidly in both directions; rows stay current with no blank or stale frames.
2. Insert and remove rows while scrolled; visible row labels and accessibility contents update.
3. Change theme and appearance with mounted views; visible colors update without reopening the story.
4. Type in a large Rust CodeEditor; text, caret, and syntax colors track each edit.
5. Edit a grid cell and move focus; the cell value and header relation remain current in the accessibility tree.
6. Open and close menu/popover stories repeatedly; popup state and item labels never lag or remain visible.
7. Resize, minimize, restore, and move the window between displays; no stale frame, crash, or lost input appears.
8. Record macOS version, display scale, each PASS/FAIL, and any stale frame or VoiceOver mismatch.
