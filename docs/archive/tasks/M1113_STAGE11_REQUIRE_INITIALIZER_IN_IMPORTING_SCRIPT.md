# M1113 — Stage 11 require initialization in the importing module script

## Contract

The Web shell must await the initializer imported from the generated Playground
module in that same module script, after the import. A call in a separate
module script cannot initialize that script's module bindings.

## Change

The Web packager now rejects an initializer call that appears before the import
or in a different module script. The focused packaging regression covers the
separate-script false positive and confirms the prior bundle is preserved.

## Evidence

The focused packaging shell test passed. `bash -n` and `sh -n` passed for the
packager. Scoped whitespace checks found no errors.

This proves a packaging guard only; no Wasm browser build or browser runtime
acceptance was performed.
