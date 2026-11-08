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
