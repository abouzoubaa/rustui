#!/usr/bin/env python3
"""Verify and stage the observed dependency notice collection and font licenses."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tempfile
from urllib.parse import unquote

FONT_LICENSES = ("FiraMono-LICENSE", "Inter-LICENSE", "NotoSans-LICENSE")


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def regular_tree(root):
    files, directories = set(), set()
    for base, dirs, names in os.walk(root, followlinks=False):
        here = Path(base)
        directories.update((here / name).relative_to(root).as_posix() for name in dirs)
        for name in list(dirs):
            if (here / name).is_symlink():
                raise ValueError(f"symlink in notice tree: {here / name}")
        for name in names:
            path = here / name
            if path.is_symlink() or not path.is_file():
                raise ValueError(f"non-regular notice input: {path}")
            files.add(path.relative_to(root).as_posix())
    return files, directories


def parent_directories(paths):
    result = set()
    for raw in paths:
        parent = PurePosixPath(raw).parent
        while str(parent) != ".":
            result.add(parent.as_posix())
            parent = parent.parent
    return result


def validate_collection(source):
    if source.is_symlink() or not source.is_dir():
        raise ValueError("notice collection must be a real directory")
    report_path = source / "report.json"
    readme_path = source / "REPORT.txt"
    if report_path.is_symlink() or readme_path.is_symlink() or not report_path.is_file() or not readme_path.is_file():
        raise ValueError("notice collection needs regular report.json and REPORT.txt")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if not isinstance(report, dict) or not isinstance(report.get("packages"), list):
        raise ValueError("notice collection report has an invalid package list")
    if report.get("schema_version") != 1 or report.get("complete_license_or_compatibility_claim") is not False:
        raise ValueError("unsupported collection report or completeness claim")
    expected = {"report.json", "REPORT.txt"}
    for package in report["packages"]:
        if not isinstance(package, dict) or not isinstance(package.get("notices"), list):
            raise ValueError("notice collection report has an invalid package entry")
        for notice in package["notices"]:
            if not isinstance(notice, dict):
                raise ValueError("notice collection report has an invalid notice entry")
            raw, checksum = notice.get("path"), notice.get("sha256")
            rel = PurePosixPath(raw or "")
            if (not raw or "\\" in raw or rel.is_absolute() or ".." in rel.parts
                    or rel.as_posix() != raw or not raw.startswith("notices/")
                    or len(checksum or "") != 64):
                raise ValueError("unsafe collection notice path or hash")
            path = source / raw
            if path.is_symlink() or not path.is_file() or digest(path) != checksum:
                raise ValueError(f"notice hash mismatch: {raw}")
            expected.add(raw)
    actual, directories = regular_tree(source)
    if actual != expected or directories != (parent_directories(expected) | {"notices"}):
        raise ValueError("notice collection contains missing or unrelated files")
    return report


def validate_destination_path(destination):
    if ".." in destination.parts:
        raise ValueError("parent traversal in packaged notice destination")
    if destination.is_symlink() or destination.parent.is_symlink():
        raise ValueError("symlink at packaged notice destination")


def validate_existing(destination, font_source):
    if not destination.exists() and not destination.is_symlink():
        return
    if destination.is_symlink() or not destination.is_dir():
        raise ValueError("existing release-notices destination is not a real directory")
    expected = {"dependency-notices/report.json", "dependency-notices/REPORT.txt"}
    report_path = destination / "dependency-notices/report.json"
    if report_path.is_symlink() or not report_path.is_file():
        raise ValueError("existing release-notices destination is unknown")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    for package in report.get("packages", []):
        for notice in package.get("notices", []):
            expected.add("dependency-notices/" + notice["path"])
    expected.update("fonts/" + name for name in FONT_LICENSES)
    actual, directories = regular_tree(destination)
    if actual != expected or directories != (parent_directories(expected) | {"dependency-notices/notices"}):
        raise ValueError("existing release-notices destination contains unknown or missing files")
    validate_collection(destination / "dependency-notices")
    for name in FONT_LICENSES:
        path = destination / "fonts" / name
        if path.is_symlink() or not path.is_file() or digest(path) != digest(font_source / name):
            raise ValueError(f"missing or changed packaged font license: {name}")


def stable_location(raw, repo_root, name, version):
    if not isinstance(raw, str) or not raw:
        return raw
    path = Path(raw)
    if path.is_absolute():
        try:
            relative = path.resolve().relative_to(repo_root.resolve()).as_posix()
        except ValueError:
            return f"cargo-source:{name}-{version}/{path.name}"
        return f"repository:{relative}"
    return raw


def stable_package_id(package_id, repo_root, name, version):
    if isinstance(package_id, str) and package_id.startswith("path+file://"):
        local, marker, fragment = package_id[len("path+file://"):].partition("#")
        stable = stable_location(unquote(local), repo_root, name, version)
        if stable.startswith("repository:"):
            stable = stable.removeprefix("repository:")
            return f"path+repository:{stable}{marker}{fragment}"
        return f"path+cargo-source:{name}-{version}{marker}{fragment}"
    return package_id


def distributable_report(report, repo_root):
    projected = dict(report)
    packages = []
    for original in report["packages"]:
        package = dict(original)
        name, version = package.get("name", "unknown"), package.get("version", "unknown")
        package["package_id"] = stable_package_id(package.get("package_id"), repo_root, name, version)
        package["manifest_path"] = stable_location(package.get("manifest_path"), repo_root, name, version)
        package.pop("source_root", None)
        package["notices"] = [
            {**notice, "source": stable_location(notice.get("source"), repo_root, name, version)}
            for notice in package.get("notices", [])
        ]
        packages.append(package)
    projected["packages"] = packages
    projected["unresolved_gaps"] = []
    for original in report.get("unresolved_gaps", []):
        gap = dict(original)
        gap["package_id"] = stable_package_id(
            gap.get("package_id"), repo_root, gap.get("name", "unknown"), gap.get("version", "unknown")
        )
        projected["unresolved_gaps"].append(gap)
    return projected


def package(source, font_source, destination, repo_root):
    report = validate_collection(source)
    if font_source.is_symlink() or not font_source.is_dir():
        raise ValueError("font license source must be a real directory")
    for name in FONT_LICENSES:
        path = font_source / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"missing embedded font license: {name}")
    source_resolved = source.resolve()
    font_resolved = font_source.resolve()
    dest_resolved = destination.absolute()
    if dest_resolved == source_resolved or source_resolved in dest_resolved.parents or dest_resolved in source_resolved.parents:
        raise ValueError("destination overlaps collection")
    if dest_resolved == font_resolved or font_resolved in dest_resolved.parents or dest_resolved in font_resolved.parents:
        raise ValueError("destination overlaps font source")
    validate_destination_path(destination.absolute())
    validate_existing(destination, font_source)
    destination.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{destination.name}.next-", dir=destination.parent))
    previous = destination.with_name(destination.name + f".previous-{os.getpid()}")
    try:
        dependency = stage / "dependency-notices"
        dependency.mkdir()
        for relative in regular_tree(source)[0]:
            target = dependency / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            if relative != "report.json":
                shutil.copyfile(source / relative, target)
        projected = distributable_report(report, repo_root)
        (dependency / "report.json").write_text(
            json.dumps(projected, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        staged_report = validate_collection(dependency)
        if staged_report != projected:
            raise ValueError("staged notice report changed")
        fonts = stage / "fonts"
        fonts.mkdir()
        for name in FONT_LICENSES:
            shutil.copyfile(font_source / name, fonts / name)
            if digest(fonts / name) != digest(font_source / name):
                raise ValueError(f"font license copy changed: {name}")
        if previous.exists() or previous.is_symlink():
            raise ValueError("refusing to overwrite an existing recovery directory")
        if destination.exists():
            os.replace(destination, previous)
        try:
            os.replace(stage, destination)
        except BaseException:
            if previous.exists():
                os.replace(previous, destination)
            raise
        if previous.exists():
            shutil.rmtree(previous)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Packaged {len(report.get('packages', []))} observed packages; unresolved gaps: {len(report.get('unresolved_gaps', []))}.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--collection", type=Path, required=True)
    parser.add_argument("--font-source", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    try:
        package(args.collection, args.font_source, args.destination, args.repo_root.resolve())
    except (OSError, ValueError, KeyError, TypeError, AttributeError, json.JSONDecodeError) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
