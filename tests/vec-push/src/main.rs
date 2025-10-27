#![feature(register_tool)]
#![register_tool(redpen)]

fn main() {
    let mut v = vec![0];
    v.push(1);
}

#[redpen::silence_panic]
pub fn dont_report() {
    let mut v = vec![0];
    v.push(1);
}
