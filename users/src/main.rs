#[macro_use]
extern crate lazy_static;

mod error;
mod handlers;
mod repositories;
mod utils;

pub use utils::prelude;

fn main() {
    println!("Hello, world!");
}
