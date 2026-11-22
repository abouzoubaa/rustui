"""Compile the safety patch's actual state tests without experimental pacing."""
from pathlib import Path
import subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[2]
def block(source,needle):
    start=source.index(needle);i=source.index('{',start)+1;depth=1
    while depth:
        depth+=(source[i]=='{')-(source[i]=='}');i+=1
    return source[start:i]
class PresentOrderSafetyTests(unittest.TestCase):
    def test_order_source_drain_dirty_retry_and_legacy_capacity(self):
        core=(ROOT/'crates/rustui-gpui-fast/src/fast/present_order.rs').read_text()
        apple=(ROOT/'crates/rustui-gpui-fast-apple/src/fast/present_order.rs').read_text()
        order='use std::{collections::HashMap,sync::OnceLock};type WindowId=u64;\n'+core[core.index('#[derive(Default)]'):core.index('thread_local!')]
        order+=block(core,'#[cfg(test)]')
        capacity='mod fast {pub mod drawable_capacity {'+(ROOT/'crates/rustui-gpui-fast-apple/src/fast/drawable_capacity.rs').read_text()+'}}\n'+block(apple,'pub(crate) fn acquisition_allowed(')+block(apple,'#[cfg(test)]')
        with tempfile.TemporaryDirectory(prefix='rustui-g5-safety-') as directory:
            for name,source in [('order',order),('capacity',capacity)]:
                rust=Path(directory)/(name+'.rs');binary=Path(directory)/name;rust.write_text(source)
                subprocess.run(['rustup','run','1.98.0','rustc','--edition=2024','--test',str(rust),'-o',str(binary)],cwd=ROOT,check=True)
                subprocess.run([str(binary)],cwd=ROOT,check=True)
if __name__=='__main__':unittest.main()
