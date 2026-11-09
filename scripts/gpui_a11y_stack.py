"""Recover the prior approved image only after verifying the accessibility layer."""
import hashlib
import importlib.util
from pathlib import Path
import subprocess


def prior_or_current(root, artifacts, relative, expected):
    root, artifacts = Path(root), Path(artifacts)
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() == expected:
        return data
    try:
        spec = importlib.util.spec_from_file_location(
            "validation_grid_a11y_patch", Path(__file__).with_name("apply_gpui_validation_grid_a11y_patch.py")
        )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        prior = module.prior_source(root, artifacts, str(relative))
        if hashlib.sha256(prior).hexdigest() == expected:
            return prior
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
        pass
    return data


def tree_prior_or_current(root, artifacts, relative, expected):
    """Read the prior approved Tree series image beneath the P2 keyboard fix."""
    import contextlib
    import io
    import tempfile
    root, artifacts, relative = Path(root), Path(artifacts), Path(relative)
    data = prior_or_current(root, artifacts, relative, expected)
    if str(relative) == "vendor/gpui-base-0.7.3/src/tree.rs" and (artifacts / "patches/approved/r15-tree-disabled-focus-v1.json").is_file():
        from approved_single_source_patch import run as replay
        from apply_r15_tree_disabled_focus_v1_patch import MANIFEST as newer_manifest, MANIFEST_SHA256 as newer_hash
        try:
            with tempfile.TemporaryDirectory(prefix="r15-tree-prior-") as temporary:
                stage = Path(temporary)
                source = stage / relative
                source.parent.mkdir(parents=True)
                source.write_bytes(data)
                with contextlib.redirect_stdout(io.StringIO()):
                    replay(stage, artifacts, newer_manifest, newer_hash, "--reverse")
                data = source.read_bytes()
        except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
            pass
    if hashlib.sha256(data).hexdigest() == expected:
        return data
    if str(relative) != "vendor/gpui-base-0.7.3/src/tree.rs":
        return data
    if not (artifacts / "patches/approved/gpui-tree-disabled-keyboard-v2.json").is_file():
        return data
    from approved_single_source_patch import run
    from apply_gpui_tree_disabled_keyboard_v2_patch import MANIFEST, MANIFEST_SHA256
    try:
        with tempfile.TemporaryDirectory(prefix="p2-tree-prior-") as temporary:
            stage = Path(temporary)
            source = stage / relative
            source.parent.mkdir(parents=True)
            source.write_bytes(data)
            with contextlib.redirect_stdout(io.StringIO()):
                run(stage, artifacts, MANIFEST, MANIFEST_SHA256, "--reverse")
            prior = source.read_bytes()
            if hashlib.sha256(prior).hexdigest() == expected:
                return prior
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
        pass
    return data


def mouse_prior_or_current(root, artifacts, relative, expected):
    """Read the prior A11Y div image beneath the P2 primary-mousedown fix."""
    import contextlib
    import io
    import tempfile

    root, artifacts, relative = Path(root), Path(artifacts), Path(relative)
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() == expected:
        return data
    if str(relative) == "vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs":
        try:
            data = grid_debug_prior_or_current(root, artifacts, relative)
        except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
            return data
        if not (artifacts / "patches/approved/r15-test-tree-disabled-v1.json").is_file():
            return data
        from approved_single_source_patch import run
        from apply_r15_test_tree_disabled_v1_patch import MANIFEST, MANIFEST_SHA256
        try:
            with tempfile.TemporaryDirectory(prefix="r15-debug-prior-") as temporary:
                stage = Path(temporary)
                source = stage / relative
                source.parent.mkdir(parents=True)
                source.write_bytes(data)
                with contextlib.redirect_stdout(io.StringIO()):
                    run(stage, artifacts, MANIFEST, MANIFEST_SHA256, "--reverse")
                prior = source.read_bytes()
                if hashlib.sha256(prior).hexdigest() == expected:
                    return prior
        except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
            pass
        return data
    if str(relative) != "vendor/gpui-pre-0.3.9/src/elements/div.rs":
        return data
    if not (artifacts / "patches/approved/gpui-primary-mousedown-click-v2.json").is_file():
        return data
    from approved_single_source_patch import run
    from apply_gpui_primary_mousedown_click_v2_patch import MANIFEST, MANIFEST_SHA256

    try:
        with tempfile.TemporaryDirectory(prefix="p2-mouse-prior-") as temporary:
            stage = Path(temporary)
            source = stage / relative
            source.parent.mkdir(parents=True)
            source.write_bytes(data)
            with contextlib.redirect_stdout(io.StringIO()):
                run(stage, artifacts, MANIFEST, MANIFEST_SHA256, "--reverse")
            prior = source.read_bytes()
            if hashlib.sha256(prior).hexdigest() == expected:
                return prior
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
        pass
    return data


def grid_debug_prior_or_current(root, artifacts, relative):
    """Recover R15's exact debug source beneath G10, with full pinned replay."""
    import contextlib
    import io
    import json
    import tempfile
    from approved_single_source_patch import run
    from apply_g10_labelled_by_tree_v1_patch import MANIFEST, MANIFEST_SHA256
    root, artifacts, relative = Path(root), Path(artifacts), Path(relative)
    data = (root / relative).read_bytes()
    if not (artifacts / MANIFEST).is_file():
        return data
    with tempfile.TemporaryDirectory(prefix="g10-grid-debug-prior-") as tmp:
        stage = Path(tmp)
        source = stage / relative
        source.parent.mkdir(parents=True)
        source.write_bytes(data)
        with contextlib.redirect_stdout(io.StringIO()):
            run(stage, artifacts, MANIFEST, MANIFEST_SHA256, "--check")
            meta = json.loads((artifacts / MANIFEST).read_text())
            if hashlib.sha256(data).hexdigest() == meta["post_sha256"]:
                run(stage, artifacts, MANIFEST, MANIFEST_SHA256, "--reverse")
        return source.read_bytes()
