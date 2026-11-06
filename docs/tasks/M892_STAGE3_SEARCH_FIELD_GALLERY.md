# M892 — Stage 3 / expand Search Field examples

The Input story now presents six Search Field reference specimens: empty, active with a clear action, recent searches with working selection and clearing, recipient badges with an editable query, editable ComboBox, and disabled search/ComboBox/Stepper controls.

At M892 recipient badges were visual only because RustUI had no public
token-entry API. M899 adds a controlled `TokenField` and wires editable token
behavior into this gallery. The catalog remains Partial for unrepresented
reference combinations. Native visual and accessibility acceptance remain
open.

Focused command passed 3 tests:

```sh
./scripts/cargo-headstart test -p rustui-playground search_field
```
