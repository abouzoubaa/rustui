extern crate block;

use block::{Block, ConcreteBlock, RcBlock};
use std::{cell::Cell, ffi::c_void, rc::Rc};

extern "C" {
    fn invoke_block(block: &Block<(i32,), i32>, value: i32) -> i32;
    fn make_block(seed: i32) -> *mut c_void;
}

struct DropCounter(Rc<Cell<usize>>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn main() {
    let drops = Rc::new(Cell::new(0));
    let capture = DropCounter(drops.clone());
    let block = ConcreteBlock::new(move |n: i32| {
        let _ = &capture;
        n + 6
    });
    assert_eq!(unsafe { invoke_block(&block, 7) }, 13);
    let copied = block.copy();
    assert_eq!(unsafe { invoke_block(&copied, 8) }, 14);
    assert_eq!(unsafe { copied.call((9,)) }, 15);
    assert_eq!(drops.get(), 0);
    drop(copied);
    assert_eq!(drops.get(), 1);

    let foreign = unsafe { RcBlock::<(i32,), i32>::new(make_block(11).cast()) };
    assert_eq!(unsafe { foreign.call((7,)) }, 18);
    drop(foreign);
    println!("C/Rust block invocation, copy and captured-value lifetime passed");
}
