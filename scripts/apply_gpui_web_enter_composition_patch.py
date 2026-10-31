#!/usr/bin/env python3
"""Approved Enter patch compatibility; composed verification uses exact full stack."""
from pathlib import Path
import argparse
import runpy


def main():
    parser=argparse.ArgumentParser(description=__doc__);group=parser.add_mutually_exclusive_group(required=True)
    for flag in ["--check","--apply","--reverse"]:group.add_argument(flag,action="store_true")
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]);args=parser.parse_args();root=args.root.resolve()
    module=runpy.run_path(str(root/"scripts/apply_gpui_web_semantic_adapter.py"));meta=module["load"](root,root);state=module["identify"](module["sources"](root,meta),meta)
    if args.check:
        if state not in ("prerequisites","post"):parser.error("approved Enter patch is not installed")
        mode="--verify-installed"
    elif args.apply:
        if state!="name":parser.error("apply the approved name prerequisite first, or use the full stack --apply-prerequisites")
        mode="--apply-prerequisites"
    else:
        if state!="prerequisites":parser.error("reverse the semantic adapter before reversing its Enter prerequisite")
        mode="--reverse-enter"
    module["run"](root,root,mode)
if __name__=="__main__":main()
