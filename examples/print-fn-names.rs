#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_public::{CrateDef, CrateItem, mir::mono::Instance, ty::FnDef};
use std::ops::ControlFlow;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let local_crate = rustc_public::local_crate();
    let crate_name = &*local_crate.name;
    for f in local_crate.fn_defs() {
        let inst_name = name_from_instance(&f);
        println!("[{crate_name}] {:33} - (instance) {inst_name}", f.name());
    }

    ControlFlow::Break(())
}

fn name_from_instance(f: &FnDef) -> String {
    Instance::try_from(CrateItem(f.def_id()))
        .map(|inst| inst.name())
        .unwrap_or_else(|err| err.to_string())
}
