# M1190 — Stage 8 Collection root accessibility identity

## Scope

Give each Collection's rendered List root a stable identity derived from its
caller-owned Collection ID, so sibling Collections do not share the same GPUI
element ID.

## Change

The Collection renderer now includes its public identity in the List root ID.
Existing List and ListItem AccessKit metadata is retained.

## Verification

Focused check passed:

```sh
./scripts/cargo-headstart test -p rustui collection_elements_emit_list_and_list_item_accesskit_metadata
```

Result: 1 passed. Native/browser interaction and accessibility acceptance
remain open.
