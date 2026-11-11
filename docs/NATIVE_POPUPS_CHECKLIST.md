# NP-10 Native popup acceptance checklist
1. Story: launch the Playground and open Popover Gallery on macOS.
2. Click “Open editor popover”; confirm it opens beside the trigger and may extend outside the parent window while staying on screen.
3. Drag and resize the parent; confirm the popup follows the trigger and remains correctly placed.
4. Scroll the owner content until the trigger leaves its visible area; confirm the popup closes.
5. Edit Notes and change the inherited value; confirm the popup works and the owner sees the updated value.
6. Click outside; confirm the popup closes and focus returns to the trigger.
7. Reopen and press Escape; confirm it closes and focus returns to the trigger.
8. Close the parent while the popup is open; confirm the popup closes.
9. Repeat near screen edges and on available displays; use Accessibility Inspector or VoiceOver to check usable names, then record macOS version, displays, and PASS/FAIL per item.
