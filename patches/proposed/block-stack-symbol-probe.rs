extern crate block;
use block::ConcreteBlock;
fn main() { let text = String::from("RustUI"); let block = ConcreteBlock::new(move |n: i32| text.len() as i32 + n); let copied = block.copy(); assert_eq!(unsafe { copied.call((7,)) }, 13); println!("block invocation and copied closure passed"); }
