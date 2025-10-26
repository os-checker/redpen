#![feature(rustc_private)]

extern crate indexmap;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

mod call_graph;
mod detect;
mod diagnostics;
mod fn_item;

use crate::{call_graph::CallGraph, detect::Detect, fn_item::FnItem};
use std::ops::ControlFlow;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let mut entries = Vec::new();
    let mut call_graph = CallGraph::default();
    let local_crate = rustc_public::local_crate();

    for f in local_crate.fn_defs() {
        let fn_item = FnItem::new(f);
        entries.push(fn_item.clone());
        call_graph.reach_in_depth(fn_item);
    }

    call_graph.sort();

    let detect = Detect::new(&call_graph, entries);
    call_graph.diagnostic(&detect);

    ControlFlow::Break(())
}
