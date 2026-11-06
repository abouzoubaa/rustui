# F1a: keyed and conditional view runtime

`Keyed(key, view)` establishes a stable identity for a view subtree. Its key is local to the nearest keyed retention scope. Reordering keyed siblings preserves retained controls under each key; removing a key releases its unseen controls at the end of that frame. Reusing a key in the same logical scope during one render is rejected, including when neither subtree creates a retained control. A plain `VStack` or `HStack` does not create a key namespace, so callers need a keyed collection owner when independent collections might reuse child keys.

`Option<V>` is a conditional view: `Some(view)` lowers the child directly, while `None` lowers GPUI's `Empty` element (`display: none`). Inactive children are not described or mounted as separate components. GPUI owns the root component lifecycle and render scheduling.

`RenderContext::with_element_scope` uses the existing retained-control scope stack and wraps the lowered child in a native GPUI element identity. The wrapper returns the child's layout ID directly and forwards prepaint and paint once, adding no layout container or accessibility role. Local scope IDs have distinct tags and length-prefixed payloads; GPUI supplies ancestry through nested wrappers. Tabs, pages, and keys therefore share the same native identity boundary as the retained-control scope.

This runtime work does not add a second state or rendering engine. Build, visual, and accessibility acceptance remain separate integration checks.
