#!/usr/bin/env python3
"""Verify the approved Web patches as one exact, read-only source stack."""

from pathlib import Path

import apply_gpui_bidi_wrapped_rows_patch as bidi
import apply_gpui_bidi_input_hit_patch as input_hit
import apply_gpui_web_semantic_adapter as semantic


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    input_meta = input_hit.load_manifest()
    input_state, current_input = input_hit.current_images(input_meta)
    if input_state != "post":
        raise ValueError("approved bidirectional input hit patch is not applied")
    before_input = input_hit.staged_images(input_meta, current_input, reverse=True)

    bidi_meta = bidi.load_manifest()
    current_bidi = {path: (root / path).read_bytes() for path in bidi.EXPECTED_FILES}
    current_bidi.update(
        (path, data) for path, data in before_input.items() if path in current_bidi
    )
    if any(
        bidi.digest(current_bidi[path]) != image["post_sha256"]
        for path, image in bidi_meta["images"].items()
    ):
        raise ValueError("approved bidirectional wrapping patch is not applied")
    before_bidi = bidi.staged_images(bidi_meta, current_bidi, reverse=True)

    semantic_meta = semantic.load(root, root)
    before_bidi_stack = semantic.sources(root, semantic_meta)
    before_bidi_stack.update(
        (path, data) for path, data in before_bidi.items() if path in before_bidi_stack
    )
    if semantic.identify(before_bidi_stack, semantic_meta) != "post":
        raise ValueError("approved semantic Web adapter is not applied")
    semantic.prove(root, root, before_bidi_stack, "post", semantic_meta)
    print("Approved bidirectional input and semantic Web patch stack verified; checkout unchanged")


if __name__ == "__main__":
    main()
