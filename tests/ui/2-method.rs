pub fn panic() {
    panic!("💥")
}

struct S {}
impl S {
    pub fn caller(&self) {
        panic();
        panic!("Second panic.")
    }
}
