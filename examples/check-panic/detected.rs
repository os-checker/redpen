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

    pub fn two_panics(&self) {
        self.b();
        let mut v = vec![0];
        v.push(1);
    }
}
