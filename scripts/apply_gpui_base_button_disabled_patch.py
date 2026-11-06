#!/usr/bin/env python3
"""Exact approved Base Button disabled accessibility patch; no revision changes."""
import argparse,hashlib,json,os,subprocess,tempfile
from pathlib import Path
MANIFEST=Path('patches/approved/gpui-base-button-disabled-accessibility-v1.json')
MANIFEST_SHA256='0d0d56328104c9d06ea5611df7c7558de773ee74d75bf831dbac07beeecb75a9'
def digest(data):return hashlib.sha256(data).hexdigest()
def run(root,artifacts,mode):
 root=root.resolve();artifacts=artifacts.resolve();data=(artifacts/MANIFEST).read_bytes()
 if digest(data)!=MANIFEST_SHA256:raise ValueError('metadata hash mismatch')
 meta=json.loads(data);patch=artifacts/meta['patch'];source=root/meta['source'];old=source.read_bytes()
 if digest(patch.read_bytes())!=meta['patch_sha256']:raise ValueError('patch hash mismatch')
 req=meta['prerequisite']
 if digest((root/req['path']).read_bytes())!=req['sha256']:raise ValueError('approved aria-disabled prerequisite source changed')
 current=digest(old)
 if current not in (meta['pre'],meta['post']):raise ValueError('unknown or partial Base Button source')
 if mode=='--verify' and current!=meta['post']:raise ValueError('exact POST required')
 with tempfile.TemporaryDirectory(prefix='gpui-button-disabled-') as folder:
  stage=Path(folder);dest=stage/meta['source'];dest.parent.mkdir(parents=True);dest.write_bytes(old)
  reverse=current==meta['post'];command=['git','apply','--whitespace=nowarn']
  if reverse:command+=['--reverse']
  subprocess.run(command+['--check',str(patch)],cwd=stage,check=True,capture_output=True)
  subprocess.run(command+[str(patch)],cwd=stage,check=True,capture_output=True)
  new=dest.read_bytes();expected=meta['pre' if reverse else 'post']
  if digest(new)!=expected:raise ValueError('exact patch replay hash mismatch')
  back=['git','apply','--whitespace=nowarn']+([] if reverse else ['--reverse'])
  subprocess.run(back+[str(patch)],cwd=stage,check=True,capture_output=True)
  if dest.read_bytes()!=old:raise ValueError('reverse replay mismatch')
 if mode in ('--apply','--reverse'):
  if (mode=='--apply')==reverse:raise ValueError('source already at requested image')
  if source.read_bytes()!=old:raise ValueError('source changed during staging')
  fd,name=tempfile.mkstemp(prefix='.gpui-button-disabled-',dir=source.parent)
  try:
   os.fchmod(fd,source.stat().st_mode & 0o777)
   with os.fdopen(fd,'wb') as output:output.write(new)
   if source.read_bytes()!=old:raise ValueError('source changed during staging')
   os.replace(name,source)
  finally:Path(name).unlink(missing_ok=True)
 print('Base Button disabled accessibility exact source/patch/replay verified')
def main():
 p=argparse.ArgumentParser();p.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1]);p.add_argument('--artifacts',type=Path)
 modes=p.add_mutually_exclusive_group(required=True)
 for mode in ('--check','--apply','--verify','--reverse'):modes.add_argument(mode,action='store_true')
 args=p.parse_args();mode=next(m for m in ('--check','--apply','--verify','--reverse') if getattr(args,m[2:].replace('-','_')))
 try:run(args.root,args.artifacts or args.root,mode)
 except (ValueError,OSError,subprocess.CalledProcessError) as error:p.exit(1,f'error: {error}\n')
if __name__=='__main__':main()
