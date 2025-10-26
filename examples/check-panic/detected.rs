#![allow(dead_code)]

pub fn a() {
    panic!("This panics!");
}

struct S;
impl S {
    fn b(&self) {
        a();
    }

    fn no_panic(&self) {}
}
