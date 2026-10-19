#!/usr/bin/env python3
"""Validate RustUI locale metadata and the supported catalog subset."""

import argparse
import json
import re
import sys
from pathlib import Path

MESSAGE_ID = re.compile(r"^[A-Za-z][A-Za-z0-9_.-]*$")


def catalog_ids(path: Path) -> set[str]:
    identifiers: set[str] = set()
    for line_number, source_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = source_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError(f"{path}:{line_number}: expected 'message-id = value'")
        identifier, value = (part.strip() for part in line.split("=", 1))
        if not MESSAGE_ID.fullmatch(identifier):
            raise ValueError(f"{path}:{line_number}: invalid message id {identifier!r}")
        if not value:
            raise ValueError(f"{path}:{line_number}: empty message value")
        if identifier in identifiers:
            raise ValueError(f"{path}:{line_number}: duplicate message id {identifier!r}")
        identifiers.add(identifier)
    if not identifiers:
        raise ValueError(f"{path}: catalog is empty")
    return identifiers


def validate_catalog_group(root: Path, name: str) -> None:
    paths = sorted(root.glob("*/" + name))
    if not paths:
        raise ValueError(f"No {name} catalogs found in {root}")
    expected = catalog_ids(paths[0])
    for path in paths[1:]:
        actual = catalog_ids(path)
        if actual != expected:
            missing = sorted(expected - actual)
            extra = sorted(actual - expected)
            raise ValueError(f"{path}: catalog mismatch; missing={missing}, extra={extra}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    repository = Path(__file__).resolve().parents[1]
    parser.add_argument("--root", type=Path, default=repository / "locales")
    parser.add_argument(
        "--lab-root",
        type=Path,
        default=repository / "apps/rustui-playground/locales",
    )
    args = parser.parse_args()

    try:
        paths = sorted(args.root.glob("*/locale.json"))
        if not paths:
            raise ValueError(f"No locale records found in {args.root}")
        for path in paths:
            data = json.loads(path.read_text(encoding="utf-8"))
            expected_fields = {
                "record_type",
                "schema_version",
                "locale",
                "direction",
                "status",
                "catalog_format",
            }
            if not isinstance(data, dict) or set(data) != expected_fields:
                raise ValueError(f"{path}: unexpected or missing locale fields")
            if (
                data["record_type"] != "rustui.locale"
                or data["schema_version"] != 1
                or data["status"] != "runtime"
                or data["catalog_format"] != "rustui-ftl-subset-v1"
                or data["locale"] != path.parent.name
                or data["direction"] not in ("ltr", "rtl")
            ):
                raise ValueError(f"{path}: invalid runtime locale record")

        validate_catalog_group(args.root, "rustui.ftl")
        validate_catalog_group(args.lab_root, "lab.ftl")
        print(f"Validated {len(paths)} runtime locale records and catalog groups.")
        return 0
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
