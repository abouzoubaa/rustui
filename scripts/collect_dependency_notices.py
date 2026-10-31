#!/usr/bin/env python3
"""Collect verbatim notices for packages observed in a successful Cargo build.

This is evidence collection, not a license compatibility or release approval tool.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import tempfile

NOTICE_NAME = re.compile(r"^(?:licen[cs]e|copying|notice|copyright|unlicense)(?:[-.][a-z0-9][a-z0-9._-]*)?$", re.I)
CODE_SUFFIXES = {".rs", ".c", ".h", ".cpp", ".py", ".js", ".ts", ".toml", ".json", ".sh"}


def notice_name(name: str) -> bool:
    return bool(NOTICE_NAME.fullmatch(name)) and Path(name).suffix.lower() not in CODE_SUFFIXES


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def package_directory(pid: str, pkg: dict, manifest_path: Path, repo_root: Path) -> str:
    if pkg.get("source") is None:
        try:
            location = manifest_path.relative_to(repo_root).as_posix()
        except ValueError:
            location = f"external:{sha256(manifest_path)}"
        identity = f"path:{location}#{pkg['name']}@{pkg['version']}"
    else:
        identity = pid
    digest = hashlib.sha256(identity.encode()).hexdigest()[:12]
    return f"{pkg['name']}-{pkg['version']}-{digest}"


def inside(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def safe_path(path: Path, roots: list[Path]) -> Path:
    # Check lexical segments before resolving links, including in-root aliases.
    # OS aliases above a source root (/tmp on macOS) remain supported.
    if ".." in path.parts:
        raise ValueError(f"parent traversal in source path: {path}")
    path = path.absolute()
    approved = {base for root in roots for base in (root.absolute(), root.resolve(strict=True))}
    for root in sorted(approved, key=lambda p: len(p.parts), reverse=True):
        try:
            relative = path.relative_to(root)
        except ValueError:
            continue
        cursor = root
        if cursor.is_symlink():
            raise ValueError(f"symlink in approved source root: {cursor}")
        for part in relative.parts:
            cursor = cursor / part
            if cursor.is_symlink():
                raise ValueError(f"symlink in approved source path: {cursor}")
        resolved = path.resolve(strict=True)
        if inside(resolved, root.resolve(strict=True)):
            return resolved
    raise ValueError(f"path is outside approved source roots: {path}")


def load_json(path: Path):
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def package_notice_files(package_root: Path) -> list[Path]:
    found: set[Path] = set()
    for current, dirs, files in os.walk(package_root, followlinks=False):
        current_path = Path(current)
        kept = []
        for name in dirs:
            p = current_path / name
            if p.is_symlink():
                raise ValueError(f"symlink in package source: {p}")
            if name.lower() in {"license", "licenses", "licence", "licences"}:
                # License directories contain files regardless of their basename.
                for sub, subdirs, subfiles in os.walk(p, followlinks=False):
                    subpath = Path(sub)
                    for d in subdirs:
                        if (subpath / d).is_symlink():
                            raise ValueError(f"symlink in license directory: {subpath / d}")
                    for file in subfiles:
                        candidate = subpath / file
                        if candidate.is_symlink():
                            raise ValueError(f"symlink notice file: {candidate}")
                        if candidate.is_file():
                            found.add(candidate)
                continue
            kept.append(name)
        dirs[:] = kept
        for name in files:
            p = current_path / name
            if p.is_symlink():
                # Only reject symlinks that could be mistaken for notice material.
                if notice_name(name):
                    raise ValueError(f"symlink notice file: {p}")
                continue
            if p.is_file() and notice_name(name):
                found.add(p)
    return sorted(found, key=lambda p: p.relative_to(package_root).as_posix())


def build_package_ids(path: Path) -> set[str]:
    ids: set[str] = set()
    success = False
    with path.open(encoding="utf-8") as stream:
        for line in stream:
            if not line.strip():
                continue
            msg = json.loads(line)
            if not isinstance(msg, dict):
                raise ValueError("Cargo build messages must be objects")
            if msg.get("reason") == "compiler-artifact":
                package_id = msg.get("package_id")
                if not isinstance(package_id, str) or not package_id:
                    raise ValueError("compiler-artifact missing package_id")
                ids.add(package_id)
            if msg.get("reason") == "build-finished":
                success = msg.get("success") is True
    if not success or not ids:
        raise ValueError("Cargo build evidence needs artifacts and a successful final build-finished message")
    return ids


def supplement_index(manifest_path: Path, supplement_root: Path, output: Path) -> dict:
    manifest = load_json(manifest_path)
    if not isinstance(manifest, dict) or manifest.get("schema_version") != 1:
        raise ValueError("supplement manifest needs schema_version 1")
    records = manifest.get("packages")
    if not isinstance(records, list):
        raise ValueError("supplement manifest needs packages")
    index = {}
    for package in records:
        if not isinstance(package, dict):
            raise ValueError("supplement package must be an object")
        if package.get("note") is not None and not isinstance(package["note"], str):
            raise ValueError("supplement review note must be text")
        if "source" not in package or not (package["source"] is None or isinstance(package["source"], str)):
            raise ValueError("supplement package needs an exact Cargo source")
        pid = (package.get("name"), package.get("version"), package.get("source"))
        digest = package.get("manifest_sha256")
        if not all(isinstance(value, str) and value for value in pid[:2]) or pid in index:
            raise ValueError("missing or duplicate supplement package identity")
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError("supplement package needs an exact manifest_sha256")
        upstream = package.get("upstream", {})
        if not isinstance(upstream, dict) or not isinstance(upstream.get("repository"), str):
            raise ValueError("supplement package needs upstream provenance")
        if not upstream["repository"] or not isinstance(upstream.get("revision"), str) or not re.fullmatch(r"[0-9a-f]{40}", upstream["revision"]):
            raise ValueError("supplement package needs repository and exact upstream revision")
        files = []
        if not isinstance(package.get("files"), list):
            raise ValueError("supplement package needs notice files")
        for item in package["files"]:
            if not isinstance(item, dict):
                raise ValueError("supplement file must be an object")
            raw, digest = item.get("path"), item.get("sha256")
            if not isinstance(raw, str) or not raw or "\\" in raw:
                raise ValueError("unsafe supplement path")
            relative = PurePosixPath(raw)
            if relative.is_absolute() or ".." in relative.parts or relative.as_posix() != raw:
                raise ValueError("unsafe or non-normalized supplement path")
            if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise ValueError("supplement file needs an exact sha256")
            url = item.get("url", "")
            if not isinstance(url, str) or not url.startswith("https://") or upstream["revision"] not in url:
                raise ValueError("supplement file needs a revision-pinned source URL")
            source = safe_path(supplement_root / relative, [supplement_root])
            if not source.is_file() or sha256(source) != digest:
                raise ValueError(f"supplement hash mismatch: {raw}")
            files.append((source, item))
        if not files:
            raise ValueError("supplement package has no notice files")
        if inside(output.resolve(), supplement_root.resolve()) or inside(supplement_root.resolve(), output.resolve()):
            raise ValueError("output directory cannot overlap supplement sources")
        index[pid] = (package, files)
    return index


def collect(args) -> dict:
    input_paths = {"metadata_sha256": args.metadata, "build_json_sha256": args.build_json}
    if args.supplement_manifest:
        input_paths["supplement_manifest_sha256"] = args.supplement_manifest
    input_hashes = {name: sha256(path) for name, path in input_paths.items()}
    repo = args.repo_root.absolute()
    root = repo.resolve(strict=True)
    approved = [repo, *args.source_root]
    metadata = load_json(args.metadata)
    if not isinstance(metadata, dict):
        raise ValueError("Cargo metadata must be an object")
    packages = metadata.get("packages")
    members = metadata.get("workspace_members")
    if not isinstance(packages, list) or not isinstance(members, list):
        raise ValueError("Cargo metadata needs packages and workspace_members")
    if not all(isinstance(member, str) for member in members):
        raise ValueError("Cargo workspace members must be package IDs")
    ids = build_package_ids(args.build_json)
    by_id = {}
    for package in packages:
        if not isinstance(package, dict):
            raise ValueError("Cargo metadata packages must be objects")
        pid = package.get("id")
        if not isinstance(pid, str) or not pid or pid in by_id:
            raise ValueError(f"invalid or duplicate Cargo metadata package id: {pid!r}")
        by_id[pid] = package
    missing = sorted(ids - by_id.keys())
    if missing:
        raise ValueError("build package IDs absent from metadata: " + ", ".join(missing))

    out = args.output.absolute()
    if out.exists() or out.is_symlink():
        raise ValueError("output must be a new directory; existing output is never replaced")
    if bool(args.supplement_manifest) != bool(args.supplement_root):
        raise ValueError("supplement manifest and root must be supplied together")
    supplement = supplement_index(args.supplement_manifest, args.supplement_root, out) if args.supplement_manifest else {}
    inputs = [args.metadata, args.build_json, root / "LICENSE"]
    if args.supplement_manifest:
        inputs += [args.supplement_manifest, args.supplement_root]
    for source in inputs:
        resolved = source.resolve()
        if inside(resolved, out.resolve()) or (source.is_dir() and inside(out.resolve(), resolved)):
            raise ValueError(f"output overlaps an input: {source}")

    # Metadata is an input even for unbuilt packages; never generate output into
    # another package's source tree merely because it was not observed in this build.
    for package in packages:
        raw_manifest = package.get("manifest_path")
        if not isinstance(raw_manifest, str) or not raw_manifest:
            raise ValueError("Cargo package needs a manifest path")
        source_dir = Path(raw_manifest).parent.resolve(strict=True)
        if inside(out.resolve(), source_dir) or inside(source_dir, out.resolve()):
            raise ValueError(f"output overlaps a package source: {source_dir}")

    prepared = []
    for pid in sorted(ids):
        pkg = by_id[pid]
        name, version = pkg.get("name"), pkg.get("version")
        if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", name):
            raise ValueError(f"invalid package name: {name!r}")
        if not isinstance(version, str) or not re.fullmatch(r"[A-Za-z0-9.+-]+", version):
            raise ValueError(f"invalid package version: {version!r}")
        if not isinstance(pkg.get("manifest_path"), str):
            raise ValueError(f"package needs a manifest path: {pid}")
        if not (pkg.get("source") is None or isinstance(pkg["source"], str)):
            raise ValueError(f"package needs an exact Cargo source: {pid}")
        manifest_path = safe_path(Path(pkg["manifest_path"]), approved)
        source_dir = manifest_path.parent
        if inside(out.resolve(), source_dir) or inside(source_dir, out.resolve()):
            raise ValueError(f"output overlaps a package source: {source_dir}")
        candidates = package_notice_files(source_dir)
        if pkg.get("license_file"):
            license_file = Path(pkg["license_file"])
            candidates.append(license_file if license_file.is_absolute() else source_dir / license_file)
        # Only actual Cargo workspace members inherit RustUI's repository license.
        if pid in members and root / "LICENSE" not in candidates:
            candidates.append(root / "LICENSE")
        sources = [(safe_path(p, approved), None) for p in sorted(set(candidates))]
        identity = (name, version, pkg.get("source"))
        if identity in supplement:
            record, files = supplement[identity]
            if any(record.get(key) != value for key, value in {
                "name": name, "version": version, "source": pkg.get("source"),
                "manifest_sha256": sha256(manifest_path),
            }.items()):
                raise ValueError(f"supplement package identity mismatch: {pid}")
            if record.get("package_id") not in (None, pid):
                raise ValueError(f"supplement package ID mismatch: {pid}")
            sources.extend((path, {"file": item, "upstream": record["upstream"],
                                   "review_note": record.get("note")}) for path, item in files)
        if any(not path.is_file() for path, _ in sources):
            raise ValueError(f"notice input is not a regular file: {pid}")
        prepared.append((pid, pkg, manifest_path, source_dir, sources))

    out.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{out.name}.next-", dir=out.parent))
    entries, gaps = [], []
    used_package_dirs = set()
    try:
        (stage / "notices").mkdir()
        for pid, pkg, manifest_path, source_dir, sources in prepared:
            if not sources:
                gaps.append({"package_id": pid, "name": pkg["name"], "version": pkg["version"],
                             "reason": "no captured LICENSE/COPYING/NOTICE files"})
            package_entries = []
            package_dir = package_directory(pid, pkg, manifest_path, root)
            if package_dir in used_package_dirs:
                raise ValueError(f"duplicate portable notice directory: {package_dir}")
            used_package_dirs.add(package_dir)
            for number, (source, provenance) in enumerate(sources, 1):
                digest = sha256(source)
                relative = Path("notices") / package_dir / f"{number:02d}-{source.name}"
                destination = stage / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, destination)
                if sha256(destination) != digest or sha256(source) != digest:
                    raise ValueError(f"notice changed while copying: {source}")
                record = {"path": relative.as_posix(), "source": str(source),
                          "sha256": digest, "size_bytes": destination.stat().st_size}
                if provenance is not None:
                    if provenance["file"]["sha256"] != digest:
                        raise ValueError(f"supplement changed after verification: {source}")
                    record["supplement"] = provenance
                package_entries.append(record)
            entries.append({"package_id": pid, "name": pkg["name"], "version": pkg["version"],
                            "license_expression": pkg.get("license"), "source": pkg.get("source"),
                            "workspace_member": pid in members, "manifest_path": str(manifest_path),
                            "source_root": str(source_dir), "manifest_sha256": sha256(manifest_path),
                            "notice_review_notes": sorted({provenance["review_note"] for _, provenance in sources
                                                           if provenance and provenance.get("review_note")}),
                            "notices": package_entries})
        if any(sha256(input_paths[name]) != digest for name, digest in input_hashes.items()):
            raise ValueError("collection inputs changed while reading")
        report = {"schema_version": 1, "status": "gaps" if gaps else "notice_files_captured",
                  "scope": "packages observed in successful Cargo build JSON, including build dependencies; not an exact linked runtime closure",
                  "complete_license_or_compatibility_claim": False,
                  "inputs": input_hashes,
                  "packages": entries, "unresolved_gaps": gaps}
        (stage / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        (stage / "REPORT.txt").write_text(
            "Dependency notice collection\n"
            "This report is not a complete license or compatibility determination.\n"
            f"Observed packages: {len(entries)}; unresolved notice gaps: {len(gaps)}.\n"
            "See report.json for package IDs, sources, hashes and gaps.\n", encoding="utf-8")
        # Reserve a fresh output exclusively; never rename or delete old output.
        out.mkdir()
        for child in stage.iterdir():
            child.rename(out / child.name)
        return report
    finally:
        shutil.rmtree(stage)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--build-json", "--build-jsonl", dest="build_json", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, action="append", default=[],
                        help="additional explicitly approved Cargo source root; repeatable")
    parser.add_argument("--supplement-manifest", type=Path)
    parser.add_argument("--supplement-root", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = collect(args)
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        parser.error(str(error))
    print(f"Collected notices for {len(report['packages'])} observed packages; "
          f"unresolved gaps: {len(report['unresolved_gaps'])}. No completeness or compatibility claim.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
