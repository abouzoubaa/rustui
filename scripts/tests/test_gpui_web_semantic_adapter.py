#!/usr/bin/env python3
"""Exact semantic stack replay, tamper rejection, and transactional rollback."""
from pathlib import Path
import contextlib, importlib.util, io, json, shutil, subprocess, tempfile, unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
SPEC=importlib.util.spec_from_file_location('semantic_adapter',ROOT/'scripts/apply_gpui_web_semantic_adapter.py')
ADAPTER=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(ADAPTER)
class SemanticLifecycle(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
  self.root=Path(self.temp.name)/'repo';self.art=Path(self.temp.name)/'artifacts';self.root.mkdir()
  shutil.copytree(ROOT/'patches',self.art/'patches');shutil.copytree(ROOT/'scripts',self.art/'scripts')
  meta=json.loads((self.art/ADAPTER.MANIFEST).read_text())
  paths=set(meta['images']['post'])|set(meta['preserved_provenance'])|set(meta['packages'])|{'Cargo.toml'}
  for path in paths:
   src=ROOT/path;dest=self.root/path
   if src.exists():dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest)
  self.meta=ADAPTER.load(self.root,self.art)
  self.assertEqual(ADAPTER.identify(ADAPTER.sources(self.root,self.meta),self.meta),'post')
 def run_mode(self,mode):
  with contextlib.redirect_stdout(io.StringIO()):ADAPTER.run(self.root,self.art,mode)
 def image(self):return ADAPTER.sources(self.root,self.meta)
 def test_full_stack_lifecycle(self):
  post=self.image();self.run_mode('--verify');self.run_mode('--reverse');self.run_mode('--check')
  self.run_mode('--reverse-enter');self.run_mode('--apply-prerequisites');self.run_mode('--reverse-prerequisites')
  self.run_mode('--apply-prerequisites');self.run_mode('--apply');self.run_mode('--verify')
  self.assertEqual(self.image(),post)
 def test_unknown_and_partial_image_rejected(self):
  path=next(p for p,v in self.meta['images']['post'].items() if v is not None)
  with (self.root/path).open('ab') as f:f.write(b'\n// foreign source\n')
  with self.assertRaises(ValueError):self.run_mode('--verify')
 def test_partial_transition_rejected(self):
  self.run_mode('--reverse');before=self.image();post=ADAPTER.transform(self.root,self.art,before,[(2,False)],self.meta,'post')
  p=next(p for p in before if before[p]!=post[p] and post[p] is not None)
  (self.root/p).write_bytes(post[p])
  with self.assertRaises(ValueError):self.run_mode('--apply')
 def test_patch_tampering_rejected(self):
  p=self.art/self.meta['ordered_patches'][2]['path'];p.write_bytes(p.read_bytes()+b'\n')
  with self.assertRaises(ValueError):self.run_mode('--verify')
 def test_manifest_tampering_rejected(self):
  p=self.art/ADAPTER.MANIFEST;p.write_bytes(p.read_bytes()+b'\n')
  with self.assertRaises(ValueError):self.run_mode('--verify')
 def test_rollback_after_second_replace_failure(self):
  self.run_mode('--reverse');before=self.image();after=ADAPTER.transform(self.root,self.art,before,[(2,False)],self.meta,'post')
  original=ADAPTER.os.replace;calls=0
  def replace(src,dst):
   nonlocal calls
   calls+=1
   if calls==2:raise OSError('injected transaction failure')
   return original(src,dst)
  with patch.object(ADAPTER.os,'replace',side_effect=replace):
   with self.assertRaises(OSError):ADAPTER.install(self.root,before,after)
  self.assertEqual(self.image(),before)
  self.assertFalse(list(self.root.rglob('.gpui-semantic-*')))
 def test_concurrent_mutation_rejected(self):
  before=self.image();p=next(p for p,v in before.items() if v is not None)
  (self.root/p).write_bytes(before[p]+b'\n')
  with self.assertRaisesRegex(ValueError,'source changed'):ADAPTER.install(self.root,before,before)
if __name__=='__main__':unittest.main()
