#!/usr/bin/env python3
"""Isolated exact PRE/POST/replay, tamper and transaction regressions."""
from pathlib import Path
import contextlib,importlib.util,io,json,shutil,tempfile,unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
SPEC=importlib.util.spec_from_file_location('disabled_roots',ROOT/'scripts/apply_gpui_base_disabled_semantic_roots.py')
PATCH=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(PATCH)
class DisabledSemanticRoots(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
  self.root=Path(self.temp.name)/'repo';self.art=Path(self.temp.name)/'artifacts';self.root.mkdir()
  meta=json.loads((ROOT/PATCH.MANIFEST).read_text())
  (self.art/'patches/approved').mkdir(parents=True)
  for name in (PATCH.MANIFEST,Path(meta['patch'])):shutil.copy2(ROOT/name,self.art/name)
  paths=set(meta['images']['pre'])|set(meta['packages'])|set(meta['preserved_provenance'])|{'Cargo.toml',meta['prerequisite']['path']}
  for path in paths:
   dest=self.root/path;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/path,dest)
  self.meta=PATCH.load(self.root,self.art)
  if PATCH.identify(self.image(),self.meta)=='pre':self.run_mode('--apply')
 def image(self):return PATCH.sources(self.root,self.meta)
 def run_mode(self,mode):
  with contextlib.redirect_stdout(io.StringIO()):PATCH.run(self.root,self.art,mode)
 def test_pre_check_apply_post_reverse(self):
  post=self.image();self.run_mode('--verify');self.run_mode('--check');self.run_mode('--reverse')
  self.assertEqual(PATCH.identify(self.image(),self.meta),'pre')
  self.run_mode('--check');self.run_mode('--apply');self.run_mode('--verify');self.assertEqual(self.image(),post)
 def test_unknown_source_rejected(self):
  p=next(iter(self.image()));(self.root/p).write_bytes((self.root/p).read_bytes()+b'\n')
  with self.assertRaises(ValueError):self.run_mode('--verify')
 def test_partial_image_rejected(self):
  post=self.image();self.run_mode('--reverse');p=next(iter(post));(self.root/p).write_bytes(post[p])
  with self.assertRaises(ValueError):self.run_mode('--apply')
 def test_patch_and_manifest_tampering_rejected(self):
  for rel in (Path(self.meta['patch']),PATCH.MANIFEST):
   p=self.art/rel;old=p.read_bytes();p.write_bytes(old+b'\n')
   with self.assertRaises(ValueError):self.run_mode('--check')
   p.write_bytes(old)
 def test_provenance_tampering_rejected(self):
  p=self.root/next(iter(self.meta['preserved_provenance']));p.write_bytes(p.read_bytes()+b'\n')
  with self.assertRaises(ValueError):self.run_mode('--check')
 def test_rollback_after_second_replace_failure(self):
  self.run_mode('--reverse');before=self.image();after=PATCH.transform(before,self.meta,self.art,False)
  original=PATCH.os.replace;calls=0
  def replace(src,dst):
   nonlocal calls
   calls+=1
   if calls==2:raise OSError('injected transaction failure')
   return original(src,dst)
  with patch.object(PATCH.os,'replace',side_effect=replace):
   with self.assertRaises(OSError):PATCH.install(self.root,before,after)
  self.assertEqual(self.image(),before);self.assertFalse(list(self.root.rglob('.gpui-disabled-roots-*')))
 def test_concurrent_source_rejected(self):
  before=self.image();p=next(iter(before));(self.root/p).write_bytes(before[p]+b'\n')
  with self.assertRaisesRegex(ValueError,'source changed'):PATCH.install(self.root,before,before)
if __name__=='__main__':unittest.main()
