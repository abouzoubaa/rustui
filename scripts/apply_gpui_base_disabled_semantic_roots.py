#!/usr/bin/env python3
"""Reproduce the approved same-version three-root disabled semantic patch."""
import argparse, hashlib, json, os, subprocess, tempfile, tomllib
from pathlib import Path
MANIFEST=Path('patches/approved/gpui-base-disabled-semantic-roots-v1.json')
MANIFEST_SHA256='22601f36962c6a6fa6356812cef4b33915c2f90f97645cd94bf481f18c6b32fb'
def digest(data): return hashlib.sha256(data).hexdigest()
def load(root, artifacts):
 data=(artifacts/MANIFEST).read_bytes()
 if digest(data)!=MANIFEST_SHA256: raise ValueError('metadata hash mismatch')
 meta=json.loads(data)
 for path,expected in meta['preserved_provenance'].items():
  if digest((root/path).read_bytes())!=expected: raise ValueError(f'upstream provenance changed: {path}')
 selected=tomllib.loads((root/'Cargo.toml').read_text()).get('patch',{}).get('crates-io',{})
 for path,expected in meta['packages'].items():
  package=tomllib.loads((root/path).read_text())['package']
  if (package.get('name'),package.get('version'))!=(expected['name'],expected['version']): raise ValueError(f'package identity changed: {path}')
  if selected.get(expected['name'],{}).get('path')!=expected['path']: raise ValueError(f'local source changed: {path}')
 prerequisite=meta['prerequisite']
 if digest((root/prerequisite['path']).read_bytes())!=prerequisite['sha256']: raise ValueError('approved aria-disabled prerequisite changed')
 if digest((artifacts/meta['patch']).read_bytes())!=meta['patch_sha256']: raise ValueError('patch artifact changed')
 return meta
def sources(root,meta): return {p:(root/p).read_bytes() for p in meta['images']['pre']}
def identify(current,meta):
 hashes={p:digest(data) for p,data in current.items()}
 for image,expected in meta['images'].items():
  if hashes==expected: return image
 raise ValueError('unknown or partial semantic root source image')
def transform(current,meta,artifacts,reverse):
 with tempfile.TemporaryDirectory(prefix='gpui-disabled-roots-proof-') as directory:
  stage=Path(directory)
  for path,data in current.items():
   dest=stage/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
  command=['git','apply','--whitespace=nowarn']+(['--reverse'] if reverse else [])
  patch=artifacts/meta['patch']
  subprocess.run(command+['--check',str(patch)],cwd=stage,check=True,capture_output=True)
  subprocess.run(command+[str(patch)],cwd=stage,check=True,capture_output=True)
  result=sources(stage,meta)
  if identify(result,meta)!=('pre' if reverse else 'post'): raise ValueError('exact replay image mismatch')
  return result
def install(root,old,new):
 if any((root/path).read_bytes()!=data for path,data in old.items()): raise ValueError('source changed during staging')
 staged={}
 def temporary(path,data):
  fd,name=tempfile.mkstemp(prefix='.gpui-disabled-roots-',dir=path.parent)
  with os.fdopen(fd,'wb') as output:
   os.fchmod(output.fileno(),path.stat().st_mode & 0o777);output.write(data)
  return Path(name)
 try:
  for path,data in new.items(): staged[path]=temporary(root/path,data)
  if any((root/path).read_bytes()!=data for path,data in old.items()): raise ValueError('source changed during staging')
  for path in new: os.replace(staged[path],root/path)
 except OSError:
  for path,data in old.items():
   rollback=temporary(root/path,data)
   try: os.replace(rollback,root/path)
   finally: rollback.unlink(missing_ok=True)
  raise
 finally:
  for path in staged.values(): path.unlink(missing_ok=True)
def run(root,artifacts,mode):
 root=root.resolve();artifacts=artifacts.resolve();meta=load(root,artifacts)
 current=sources(root,meta);state=identify(current,meta);reverse=state=='post'
 if mode=='--verify' and state!='post': raise ValueError('exact POST required')
 if mode=='--apply' and state!='pre': raise ValueError('apply requires exact PRE')
 if mode=='--reverse' and state!='post': raise ValueError('reverse requires exact POST')
 changed=transform(current,meta,artifacts,reverse)
 if transform(changed,meta,artifacts,not reverse)!=current: raise ValueError('reverse replay mismatch')
 if mode in ('--apply','--reverse'): install(root,current,changed)
 print('Disabled semantic roots exact source/provenance/apply/reverse verified')
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1]);parser.add_argument('--artifacts',type=Path)
 modes=parser.add_mutually_exclusive_group(required=True)
 for mode in ('--check','--apply','--verify','--reverse'): modes.add_argument(mode,action='store_true')
 args=parser.parse_args();mode=next(m for m in ('--check','--apply','--verify','--reverse') if getattr(args,m[2:]))
 try: run(args.root,args.artifacts or args.root,mode)
 except (ValueError,OSError,subprocess.CalledProcessError) as error: parser.exit(1,f'error: {error}\n')
if __name__=='__main__': main()
