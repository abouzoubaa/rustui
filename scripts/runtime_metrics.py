"""Shared, small JSON output contract for external runtime measurements."""

import json
import os
from pathlib import Path


SCHEMA_VERSION = 1
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "target/headstart/performance"


def validate_output_path(path: Path) -> Path:
    """Accept a direct JSON child of the performance directory, without symlinks."""
    path = Path(os.path.abspath(path.expanduser()))
    if path.parent != OUTPUT_DIR or path.suffix != ".json" or path.name in {".", ".."}:
        raise ValueError(f"output must be a JSON file directly in {OUTPUT_DIR}")
    root = OUTPUT_DIR.parents[2]
    for component in [root, root / "target", root / "target/headstart", OUTPUT_DIR, path]:
        if component.is_symlink():
            raise ValueError(f"output path contains a symlink: {component}")
    return path


def default_output_path(run_id: str, suffix: str) -> Path:
    return validate_output_path(OUTPUT_DIR / f"{suffix}-{run_id}.json")


def write_json_exclusive(path: Path, document: dict) -> None:
    """Write one finite JSON document, refusing an existing output file."""
    path = validate_output_path(path)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    validate_output_path(path)
    directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    directory_fd = os.open(OUTPUT_DIR, directory_flags)
    try:
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
        fd = os.open(path.name, flags, 0o600, dir_fd=directory_fd)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as stream:
                json.dump(document, stream, indent=2, sort_keys=True, allow_nan=False)
                stream.write("\n")
        except BaseException:
            os.unlink(path.name, dir_fd=directory_fd)
            raise
    finally:
        os.close(directory_fd)
