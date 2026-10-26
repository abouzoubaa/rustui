#!/usr/bin/env python3
"""Verify the approved Web patches as one exact, read-only source stack."""

from pathlib import Path

import apply_gpui_bidi_wrapped_rows_patch as bidi
import apply_gpui_web_semantic_adapter as semantic


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    bidi_meta = bidi.load_manifest()
    state, current = bidi.current_images(bidi_meta)
    if state != "post":
        raise ValueError("approved bidirectional wrapping patch is not applied")
    before_bidi = bidi.staged_images(bidi_meta, current, reverse=True)

    semantic_meta = semantic.load(root, root)
    before_bidi_stack = semantic.sources(root, semantic_meta)
    before_bidi_stack.update(
        (path, data) for path, data in before_bidi.items() if path in before_bidi_stack
    )
    if semantic.identify(before_bidi_stack, semantic_meta) != "post":
        raise ValueError("approved semantic Web adapter is not applied")
    semantic.prove(root, root, before_bidi_stack, "post", semantic_meta)
    print("Approved bidirectional and semantic Web patch stack verified; checkout unchanged")


if __name__ == "__main__":
    main()
