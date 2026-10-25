# D1c inspector string property values

## M281 — Stage 9 / D1c

The schema inspector previously initialized each editable string field from the
shared Text preview value. When mounting Icon, the preview rendered and
exported `accessibility_label = Search` while the inspector displayed the
previous `Preview text` value.

String inspector bindings now initialize from the active preview snapshot and
read the property value from that descriptor. Reads verify that their binding
lease still matches the active preview generation; a stale lease falls back to
the previous local value instead of reading a same-named property from a new
preview.

The Icon regression covers the default `Search` value, an imported `Find item`
value, and a stale-generation read. The Playground library suite passed 46/46;
an isolated locked/offline all-targets check and Playground build passed, as did
rustfmt and scoped diff checks. In the rebuilt disposable Playground, the
inspector field showed `Search`, and the native AX tree exposed the rendered
image as `Search`.

Evidence:

- `target/headstart/m273-icon-native-library-tests.log`
- `target/headstart/m273-icon-native-all-targets.log`
- `target/headstart/m273-icon-native-build.log`
- `target/headstart/evidence/m273-icon-native-acceptance-20261020.json`

Keyboard/focus, VoiceOver, localization, theme, RTL, and cross-platform
acceptance remain open.
