#!/usr/bin/env python3
"""Copy the source consumer into a new directory, still using this checkout."""

import argparse
import json
import re
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="new directory for the consumer")
    destination = parser.parse_args().destination.absolute()
    example = Path(__file__).resolve().parent
    root = example.parent.parent

    manifest = (example / "Cargo.toml").read_text(encoding="utf-8")

    def absolute_dependency(match):
        dependency = (example / match.group(1)).resolve(strict=True)
        if not dependency.is_relative_to(root) or not (dependency / "Cargo.toml").is_file():
            raise ValueError(f"invalid local dependency: {dependency}")
        return "path = " + json.dumps(str(dependency))

    manifest, count = re.subn(r'path\s*=\s*"([^"]+)"', absolute_dependency, manifest)
    if not count:
        raise ValueError("consumer manifest has no local dependencies")
    source = (example / "src/main.rs").read_text(encoding="utf-8")
    theme_path = "../../../themes/macos27.json"
    if source.count(theme_path) != 1:
        raise ValueError("consumer theme path changed")
    source = source.replace(theme_path, "../macos27.json")
    lockfile = (example / "Cargo.lock").read_bytes()
    theme = (root / "themes/macos27.json").read_bytes()
    cargo_config = (
        '[source.crates-io]\nreplace-with = "vendored-sources"\n'
        '[source.vendored-sources]\ndirectory = ' + json.dumps(str(root / "vendor")) + "\n"
    )

    # Exclusive creation protects existing user files. A failed copy is left
    # for inspection rather than deleting a directory the user may have edited.
    try:
        destination.mkdir()
    except FileExistsError:
        parser.error(f"destination already exists: {destination}")
    (destination / "src").mkdir()
    (destination / ".cargo").mkdir()
    (destination / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (destination / "Cargo.lock").write_bytes(lockfile)
    (destination / "src/main.rs").write_text(source, encoding="utf-8")
    (destination / "macos27.json").write_bytes(theme)
    (destination / ".cargo/config.toml").write_text(cargo_config, encoding="utf-8")
    print(f"Created {destination}; this consumer still depends on {root}.")


if __name__ == "__main__":
    main()
