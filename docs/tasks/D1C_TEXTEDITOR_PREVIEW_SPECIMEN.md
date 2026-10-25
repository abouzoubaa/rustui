# D1c TextEditor preview specimen

The canonical compiled-preview gallery includes an opt-in public RustUI
`TextEditor` specimen. Its descriptor stores a string `value` and a string
`placeholder`; the initial value contains two lines so the multiline control is
visible immediately. The editor uses a fixed accessible name and the
generation-stamped binding feedback path already used by the gallery.

The specimen's event map is empty under the selected signal-only preview
contract. The existing descriptor codec carries both properties through
normalized export and import. The property inspector edits each schema string
through its existing string editor; a value containing line breaks can also be
imported from the descriptor.

## Validation

Focused source test and isolated locked/offline Playground all-targets check are
recorded at:

- `target/headstart/evidence/d1c-texteditor-preview-tests-20261020.log`
- `target/headstart/evidence/d1c-texteditor-preview-check-20261020.log`

These checks establish source and compilation behavior only. No GUI acceptance
was run; native multiline editing, accessibility, locale/theme behavior, and
cross-platform behavior remain open.
