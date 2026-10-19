#!/usr/bin/env python3
"""Validate RustUI theme planning records; runtime packs use the Rust loader."""

import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plans-only",
        action="store_true",
        help="Validate planning records, not runtime packs",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "themes",
    )
    args = parser.parse_args()
    if not args.plans_only:
        parser.error("Use --plans-only; runtime packs are validated by RustUI's loader")

    try:
        paths = sorted(args.root.glob("*.json"))
        if not paths:
            raise ValueError(f"No theme JSON files found in {args.root}")

        count = 0
        runtime_count = 0
        for path in paths:
            if path.name == "theme.schema.json":
                continue
            data = json.loads(path.read_text(encoding="utf-8"))
            if not isinstance(data, dict):
                raise ValueError(f"{path}: expected an object")
            if (
                data.get("format") == "rustui-appearance"
                and data.get("format_version") == 1
            ):
                runtime_count += 1
                continue
            if set(data) != {
                "record_type",
                "plan_version",
                "id",
                "status",
                "runtime_schema",
            }:
                raise ValueError(f"{path}: unexpected or missing planning fields")
            if (
                data.get("record_type") != "rustui.theme-plan"
                or type(data.get("plan_version")) is not int
                or data["plan_version"] != 1
                or data.get("status") != "planned"
            ):
                raise ValueError(f"{path}: unsupported planning record")
            if data.get("id") != path.stem or data.get("runtime_schema") is not None:
                raise ValueError(f"{path}: invalid planning identity/schema")
            count += 1

        if count == 0:
            raise ValueError(f"No theme planning records found in {args.root}")
        print(
            f"Validated {count} planning records; skipped {runtime_count} runtime packs "
            "(use RustUI's loader to validate those)."
        )
        return 0
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
