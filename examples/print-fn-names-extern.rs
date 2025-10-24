// smir-fn-name/src/main.rs
#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_public::{mir::mono::Instance, ty::FnDef, CrateDef, CrateItem};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&args, || {
        analyze();
        ControlFlow::<(), ()>::Break(())
    });
}

fn analyze() {
    let local_crate = rustc_public::local_crate();
    let crate_name = &*local_crate.name;
    for f in local_crate.fn_defs() {
        let inst_name = name_from_instance(&f);
        println!(
            "(local ) [{crate_name:8}] {:30} - (instance) {inst_name}",
            f.name()
        );
    }

    for extern_krate in rustc_public::external_crates() {
        let root = &*extern_krate.name;
        for f in extern_krate.fn_defs().iter().take(2) {
            let inst_name = name_from_instance(f);
            println!(
                "(extern) [{root:8}] {:30} - (instance) {inst_name}",
                f.name()
            );
        }
    }
}

fn name_from_instance(f: &FnDef) -> String {
    Instance::try_from(CrateItem(f.def_id()))
        .map(|inst| inst.name())
        .unwrap_or_else(|err| err.to_string())
}
