#!/usr/bin/env python3
"""Check artifact reuse in a successful Cargo --message-format=json build log."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit


MAX_LOG_BYTES = 64 * 1024 * 1024


def read_log(path):
    with path.open("rb") as source:
        log_bytes = source.read(MAX_LOG_BYTES + 1)
    if len(log_bytes) > MAX_LOG_BYTES:
        raise ValueError(f"Cargo JSON log exceeds {MAX_LOG_BYTES}-byte limit")
    return log_bytes


def package_name(package_id):
    identifier = urlsplit(package_id)
    if "@" in identifier.fragment:
        return identifier.fragment.rsplit("@", 1)[0]
    # Cargo omits the name when it matches the URL's last segment.
    return unquote(identifier.path.rstrip("/").rsplit("/", 1)[-1])


def compiler_artifact(message, number):
    package_id = message.get("package_id")
    target = message.get("target")
    profile = message.get("profile")
    filenames = message.get("filenames")
    fresh = message.get("fresh")
    if not isinstance(package_id, str) or not package_id:
        raise ValueError(f"Line {number} has an invalid compiler-artifact package_id")
    if not isinstance(target, dict) or not isinstance(target.get("name"), str):
        raise ValueError(f"Line {number} has an invalid compiler-artifact target")
    if not isinstance(target.get("kind"), list) or not all(
        isinstance(kind, str) for kind in target["kind"]
    ):
        raise ValueError(f"Line {number} has invalid compiler-artifact target kinds")
    if not isinstance(profile, dict):
        raise ValueError(f"Line {number} has an invalid compiler-artifact profile")
    if not isinstance(filenames, list) or not all(
        isinstance(filename, str) for filename in filenames
    ):
        raise ValueError(f"Line {number} has invalid compiler-artifact filenames")
    if not isinstance(fresh, bool):
        raise ValueError(f"Line {number} has an invalid compiler-artifact fresh value")
    return {
        "package_id": package_id,
        "target_name": target["name"],
        "target_kind": target["kind"],
        "profile": profile,
        "filenames": filenames,
        "fresh": fresh,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--package", action="append", dest="packages",
                        help="Required package; repeat as needed. Default: gpui-pre, gpui-pre-platform, gpui-component.")
    parser.add_argument("--report", type=Path,
                        help="Write structured JSON evidence to PATH.")
    args = parser.parse_args()
    required = sorted(set(args.packages or ["gpui-pre", "gpui-pre-platform", "gpui-component"]))
    if not all(required):
        print("Cannot verify reuse: package names must not be empty", file=sys.stderr)
        return 2

    try:
        log_path = args.log.resolve()
        if args.report is not None:
            report_path = args.report.resolve()
            same_file = args.report.exists() and args.log.exists() and args.report.samefile(args.log)
            if report_path == log_path or same_file:
                raise ValueError("report path identifies the input log")
        log_bytes = read_log(args.log)
        log_text = log_bytes.decode("utf-8")
        artifacts_by_package = {name: [] for name in required}
        finished = []
        for number, line in enumerate(log_text.splitlines(), 1):
            if not line.strip():
                continue
            try:
                message = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError(f"Line {number} is not Cargo JSON: {error}") from error
            if not isinstance(message, dict):
                raise ValueError(f"Line {number} is not a Cargo message object")
            if finished:
                raise ValueError(f"Line {number} follows build-finished")
            if message.get("reason") == "build-finished":
                success = message.get("success")
                if not isinstance(success, bool):
                    raise ValueError(f"Line {number} has an invalid build-finished success value")
                finished.append(success)
            elif message.get("reason") == "compiler-artifact":
                artifact = compiler_artifact(message, number)
                name = package_name(artifact["package_id"])
                if name in artifacts_by_package:
                    artifacts_by_package[name].append(artifact)

        if finished != [True]:
            raise ValueError("Expected exactly one successful build-finished event")

        packages = {}
        for name, artifacts in artifacts_by_package.items():
            all_fresh = bool(artifacts) and all(artifact["fresh"] for artifact in artifacts)
            packages[name] = {
                "status": "all_fresh" if all_fresh else (
                    "rebuilt" if artifacts else "missing"
                ),
                "all_fresh": all_fresh,
                "artifacts": artifacts,
            }

        overall_reuse = all(package["all_fresh"] for package in packages.values())
        if args.report is not None:
            report = {
                "schema_version": 1,
                "log_path": str(log_path),
                "log_sha256": hashlib.sha256(log_bytes).hexdigest(),
                "successful_build": True,
                "requested_packages": required,
                "packages": packages,
                "overall_reuse": "confirmed" if overall_reuse else "not_confirmed",
            }
            args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

        for name, package in packages.items():
            print(f"{name}: {'all artifacts fresh' if package['all_fresh'] else 'missing or rebuilt'}")
        if not overall_reuse:
            return 1
        print("Reuse confirmed for the requested packages; no speedup is inferred.")
        return 0
    except (OSError, ValueError, TypeError, AttributeError, RuntimeError) as error:
        print(f"Cannot verify reuse: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
