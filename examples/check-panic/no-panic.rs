//! cargo run --example check-panics -- examples/check-panic/no-panic.rs --crate-type=lib
#![allow(dead_code)]
pub fn a() {}

struct S;
impl S {
    fn b(&self) {}
}
