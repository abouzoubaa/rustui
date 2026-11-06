# M1103 — Stage 9: clear secure preview state on component type changes

`CompiledPreviewGallery` now clears its ephemeral secure-field value after a
successful descriptor import switches the preview to another component. The
existing focused secure-field regression now imports a text-field descriptor
and verifies the secret value is cleared and the preview type changes.

The focused Playground library test passed (1 test). Broader native inspector
acceptance, including selection, IME, and retention without blur, remains open.
