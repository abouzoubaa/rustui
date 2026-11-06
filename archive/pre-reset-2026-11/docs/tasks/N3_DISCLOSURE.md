# N3: Controlled Disclosure

`Disclosure(id, label, is_open, content)` is a labeled, controlled expansion
control with an explicit nonempty identity and a `Binding<bool>`. The existing
themed GPUI Components Button owns pointer and Enter/Space activation. Its parent
GPUI Group exposes expanded state to accessibility. The pinned GPUI Components
`Collapsible` mounts the content below the trigger only while open. The trigger
remains visible while disabled; disabled activation cannot write the owner binding.

RustUI lowers `content` only while open and scopes it under the disclosure ID.
The owner host's normal retention sweep releases nested components and retained
controls after closing. Reopening creates those descendants again. The bound
owner value may also open or close the content externally. This slice has no
animation or custom trigger API.

The Playground example covers trigger, external open/close, disabled state,
owner-bound text input, and a stable-key child with its own counter. The
`Observe` control displays child mount/release counts after interaction.

The bounded M38 native Review observed the owner note `Kept note` persist after
collapse and external reopen while the child counter reset from 1 to 0. The
mount/release display advanced from 1/1 after the first collapse to 2/2 after
external close. Disabled trigger clicks left both the expanded and collapsed
states unchanged; external close still hid content while disabled. Evidence:
`target/headstart/milestone38-disclosure-gui.json`, using the Review app with
SHA-256 `48bbf1b10a43ccbd476780c5c01b90470ad3293bf30bfbfe459797b835ec1792`.

The parent's expanded accessibility state was not visible in the inspected
native tree. Keyboard activation, focus, and full screenreader behavior remain
unverified. The native observations accept only the exercised pointer and owner
lifecycle slice.
