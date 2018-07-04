#![feature(stdsimd)]

extern crate faster;

#[macro_use]
mod macros;

pub use self::vlist::{Cons, Nil};
mod vlist;

pub use self::packed::{Packed, Packable};
pub mod packed;

//mod iter;

fn main() {
    println!("Hello, world!");
}
