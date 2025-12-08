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

use crate::{call_graph::CallGraph, detect::Detect, diagnostics::SourceCode, fn_item::FnItem};
use rustc_middle::ty::TyCtxt;
use rustc_public::CrateDef;
use std::ops::ControlFlow;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run_with_tcx!(&rustc_args, analysis);
}

fn analysis(tcx: TyCtxt) -> ControlFlow<(), ()> {
    let mut entries = Vec::new();
    let mut call_graph = CallGraph::default();
    let local_crate = rustc_public::local_crate();

    for f in local_crate.fn_defs() {
        let fn_item = FnItem::new(f);
        call_graph.reach_in_depth(fn_item.clone());

        // When a top-level function is tagged, don't treat it as an entry item to report.
        let mut push_entry = true;
        for attr in f.all_tool_attrs() {
            if attr.as_str().trim() == "#[redpen::silence_panic]" {
                push_entry = false;
            }
        }
        if push_entry {
            entries.push(fn_item);
        }
    }

    call_graph.sort();

    let detect = Detect::new(&call_graph, entries);
    let spots = call_graph.analyze(&detect, tcx);

    if !spots.is_empty() {
        let src = SourceCode::new(tcx);
        spots.emit(&src);
    }

    ControlFlow::Continue(())
}
