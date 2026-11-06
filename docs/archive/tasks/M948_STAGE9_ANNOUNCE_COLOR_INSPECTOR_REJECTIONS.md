# M948 — Stage 9 / announce rejected ColorPicker inspector values

The preview inspector emitted `Color channels must be finite values from 0 to 1`
for invalid color input, while its polite live-region filter matched a misspelled
prefix (`must finite`). As a result, this rejected edit was visible in status
text but not announced. The filter now matches the emitted status. A focused
regression exercises the actual invalid write, confirms the status is announced,
and confirms the mounted preview value and stamp remain unchanged.

Focused validation: `./scripts/cargo-headstart test -p rustui-playground color_picker_inspector_rejection_is_announced_without_changing_preview`.
Native screen-reader acceptance remains part of broader Stage 9 validation.
