# M961 — Stage 9 / verify Dialog body semantics

The focused Dialog regression now inspects the emitted typed semantic body
element and proves it has `Role::Dialog` and the expected accessible label. It
passed 1/1. This does not prove semantics or focus ownership on the pinned
GPUI Components Dialog host; native accessibility acceptance remains open.
