# M466: Stable explicit Text element identity

`Text(...).id(identity)` assigns a caller-provided identity within the active
render owner. Geometry readers and data-grid cells keep their existing text
prefixes, so equal caller IDs in different owners or cells remain distinct.
Without `.id(...)`, Text keeps its positional element ID. Rendering always
advances the sibling text slot, including when an explicit identity is set.

This identity supports stable GPUI element identity only. It does not promise
retained component state, stable accessibility identifiers, or cross-platform
accessibility behavior.
