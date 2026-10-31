#!/usr/bin/env python3
"""Exact same-version Web semantic/protected-IME stack; no Cargo or app execution."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

MANIFEST = Path("patches/approved/gpui-web-semantic-adapter-v9.json")
MANIFEST_SHA256 = "bc0cb5baa1e6734f9e61a2d33d93267140ddf6b9241eee59c62525e4d7f7fe40"


def digest(data): return hashlib.sha256(data).hexdigest()


def read_source(root, relative):
    path=root/relative
    if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"nonlocal or symlinked source: {relative}")
    return path.read_bytes() if path.is_file() else None


def artifact(root, artifacts, relative):
    path=artifacts/relative
    return path if path.is_file() else root/relative


def load(root, artifacts):
    path=artifact(root,artifacts,MANIFEST)
    data=path.read_bytes()
    if digest(data)!=MANIFEST_SHA256: raise ValueError("semantic adapter metadata hash mismatch")
    meta=json.loads(data)
    for path,expected in meta["preserved_provenance"].items():
        if digest((root/path).read_bytes())!=expected: raise ValueError(f"changed upstream provenance: {path}")
    selected=tomllib.loads((root/"Cargo.toml").read_text()).get("patch",{}).get("crates-io",{})
    for path,expected in meta["packages"].items():
        package=tomllib.loads((root/path).read_text())["package"]
        if (package.get("name"),package.get("version"))!=(expected["name"],expected["version"]):raise ValueError(f"changed package identity: {path}")
        if selected.get(expected["name"],{}).get("path")!=expected["path"]:raise ValueError(f"changed local source selection: {expected['name']}")
    for patch in meta["ordered_patches"]+[meta["keyboard_baseline"]]:
        if digest(artifact(root,artifacts,patch["path"]).read_bytes())!=patch["sha256"]:raise ValueError(f"changed patch artifact: {patch['path']}")
    return meta


def sources(root,meta): return {relative:read_source(root,relative) for relative in meta["images"]["post"]}


def hashes(cache): return {relative:digest(data) if data is not None else None for relative,data in cache.items()}


def identify(cache,meta):
    current=hashes(cache)
    for name,expected in meta["images"].items():
        if current==expected:return name
    raise ValueError("unknown or partial Web source image; no source writes performed")


def write_cache(stage,cache):
    for relative,data in cache.items():
        path=stage/relative
        if data is None:path.unlink(missing_ok=True)
        else:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)


def git_patch(stage,path,reverse=False):
    command=["git","apply","--unsafe-paths"]+(["-R"] if reverse else [])
    result=subprocess.run(command+["--check",str(path.resolve())],cwd=stage,capture_output=True,text=True)
    if result.returncode:raise ValueError(f"exact patch check failed: {result.stderr.strip()}")
    result=subprocess.run(command+[str(path.resolve())],cwd=stage,capture_output=True,text=True)
    if result.returncode:raise ValueError(f"exact patch application failed: {result.stderr.strip()}")


def transform(root,artifacts,cache,steps,meta,expected):
    with tempfile.TemporaryDirectory(prefix="rustui-web-semantic-stack-") as temporary:
        stage=Path(temporary);write_cache(stage,cache)
        for index,reverse in steps:git_patch(stage,artifact(root,artifacts,meta["ordered_patches"][index]["path"]),reverse)
        result={relative:read_source(stage,relative) for relative in cache}
    if hashes(result)!=meta["images"][expected]:raise ValueError(f"staged {expected} source hashes differ")
    return result


def prove(root,artifacts,cache,state,meta):
    reverse={"post":[(2,True),(1,True),(0,True)],"prerequisites":[(1,True),(0,True)],"name":[(0,True)],"base":[]}[state]
    base=transform(root,artifacts,cache,reverse,meta,"base")
    baseline=meta["keyboard_baseline"]
    for relative,expected in baseline["sources"].items():
        if digest(base[relative])!=expected["post"]:raise ValueError(f"not the approved M855 keyboard baseline: {relative}")
    with tempfile.TemporaryDirectory(prefix="rustui-web-m855-proof-") as temporary:
        stage=Path(temporary);write_cache(stage,base)
        path=artifact(root,artifacts,baseline["path"])
        git_patch(stage,path,True)
        for relative,expected in baseline["sources"].items():
            if digest((stage/relative).read_bytes())!=expected["pre"]:raise ValueError(f"M855 pristine reverse mismatch: {relative}")
        git_patch(stage,path)
        for relative,expected in baseline["sources"].items():
            if digest((stage/relative).read_bytes())!=expected["post"]:raise ValueError(f"M855 replay mismatch: {relative}")
    forward={"base":[],"name":[(0,False)],"prerequisites":[(0,False),(1,False)],"post":[(0,False),(1,False),(2,False)]}[state]
    restored=transform(root,artifacts,base,forward,meta,state)
    if restored!=cache:raise ValueError("composed Web stack did not round-trip exactly")


def install(root,old,new):
    # A concurrent mutation must not be overwritten after staging.
    if any(read_source(root,path)!=data for path,data in old.items()):raise ValueError("source changed during staging")
    changed={path:data for path,data in new.items() if old[path]!=data}
    staged={}
    def temporary(path,data):
        path.parent.mkdir(parents=True,exist_ok=True)
        fd,name=tempfile.mkstemp(prefix=".gpui-semantic-",dir=path.parent)
        with os.fdopen(fd,"wb") as output:output.write(data)
        return Path(name)
    try:
        for relative,data in changed.items():
            if data is not None:staged[relative]=temporary(root/relative,data)
        for relative,data in changed.items():
            if data is None:(root/relative).unlink()
            else:os.replace(staged[relative],root/relative)
    except OSError:
        for relative in changed:
            if old[relative] is None:(root/relative).unlink(missing_ok=True)
            else:
                rollback=temporary(root/relative,old[relative]);os.replace(rollback,root/relative)
        raise
    finally:
        for path in staged.values():path.unlink(missing_ok=True)


def run(root,artifacts,mode):
    root=root.resolve();artifacts=artifacts.resolve();meta=load(root,artifacts)
    current=sources(root,meta);state=identify(current,meta)
    if mode=="--verify-installed":
        prove(root,artifacts,current,state,meta);print(f"Exact Web {state} image and composed M855 baseline verified; checkout unchanged");return
    if mode=="--verify":
        if state!="post":raise ValueError("--verify requires exact adapter POST")
        prove(root,artifacts,current,state,meta);print("Exact semantic/protected-IME POST and composed prerequisites verified; checkout unchanged");return
    if mode=="--check":
        if state!="prerequisites":raise ValueError("--check requires exact approved name/Enter prerequisites")
        post=transform(root,artifacts,current,[(2,False)],meta,"post");prove(root,artifacts,post,"post",meta)
        restored=transform(root,artifacts,post,[(2,True)],meta,"prerequisites")
        if restored!=current:raise ValueError("adapter reverse mismatch")
        print("Semantic/protected-IME PRE/apply/POST/reverse and M855 composition verified; checkout unchanged");return
    if mode=="--apply-prerequisites":
        if state not in ("base","name"):raise ValueError("prerequisites require exact M855 base or name-only image")
        steps=[(0,False),(1,False)] if state=="base" else [(1,False)]
        updated=transform(root,artifacts,current,steps,meta,"prerequisites")
    elif mode=="--apply":
        if state!="prerequisites":raise ValueError("adapter apply requires exact approved name/Enter prerequisites")
        updated=transform(root,artifacts,current,[(2,False)],meta,"post")
    elif mode=="--reverse":
        if state!="post":raise ValueError("adapter reverse requires exact POST")
        updated=transform(root,artifacts,current,[(2,True)],meta,"prerequisites")
    elif mode=="--reverse-enter":
        if state!="prerequisites":raise ValueError("Enter reverse requires exact prerequisite image")
        updated=transform(root,artifacts,current,[(1,True)],meta,"name")
    elif mode=="--reverse-prerequisites":
        if state!="prerequisites":raise ValueError("prerequisite reverse requires exact prerequisite image")
        updated=transform(root,artifacts,current,[(1,True),(0,True)],meta,"base")
    else:raise ValueError("unknown operation")
    prove(root,artifacts,updated,identify(updated,meta),meta);install(root,current,updated)
    print(f"Exact Web stack {mode[2:]} completed; source hashes verified")


def main():
    parser=argparse.ArgumentParser(description=__doc__);group=parser.add_mutually_exclusive_group(required=True)
    for flag in ["--check","--verify","--verify-installed","--apply-prerequisites","--apply","--reverse","--reverse-enter","--reverse-prerequisites"]:group.add_argument(flag,action="store_true")
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]);parser.add_argument("--artifacts",type=Path)
    args=parser.parse_args();mode=next(flag for flag in ["--check","--verify","--verify-installed","--apply-prerequisites","--apply","--reverse","--reverse-enter","--reverse-prerequisites"] if getattr(args,flag[2:].replace('-','_')))
    try:run(args.root,args.artifacts or args.root,mode)
    except (OSError,ValueError,KeyError,tomllib.TOMLDecodeError) as error:raise SystemExit(f"error: {error}") from error
if __name__=="__main__":main()
