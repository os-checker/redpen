#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_public::CrateDef;
use std::ops::ControlFlow;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let local_crate = rustc_public::local_crate();
    for f in local_crate.fn_defs() {
        let fn_name = f.name();
        if fn_name == "main" {
            if let Some(body) = f.body() {
                dbg!(body);
            }

            break;
        }
    }

    ControlFlow::Break(())
}
