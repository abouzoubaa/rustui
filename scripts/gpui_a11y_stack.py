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
