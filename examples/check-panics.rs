//! Check panic spots.
//!
//! `cargo run --example check-panics -- examples/vec-push/vec-push.rs`
//!
//! ```rust,ignore
//! ```
#![feature(rustc_private)]

extern crate indexmap;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use indexmap::{IndexMap, IndexSet};
use rustc_public::{
    CrateDef,
    mir::{MirVisitor, visit::Location},
    ty::{FnDef, RigidTy, Ty, TyKind},
};
use std::{fmt, ops::ControlFlow, rc::Rc};

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let mut call_graph = CallGraph::default();
    let local_crate = rustc_public::local_crate();

    for f in local_crate.fn_defs() {
        call_graph.reach_in_depth(f.into());
    }

    call_graph.sort();
    call_graph.print();
    ControlFlow::Break(())
}

#[derive(Debug, Default)]
struct CallGraph {
    edges: IndexMap<FnItem, Nodes>,
    back_edges: IndexMap<FnItem, Nodes>,
}

impl CallGraph {
    /// Reach crate function items as entries.
    fn reach_in_depth(&mut self, fn_item: FnItem) {
        if self.edges.contains_key(&fn_item) {
            // The fn item has been reached before.
            return;
        }

        let mut nodes = Nodes::default();
        if let Some(body) = fn_item.def.body() {
            nodes.visit_body(&body);
        }

        // Add direct callees on callees.
        let callees: IndexSet<_> = nodes.set.iter().cloned().collect();

        // Add reverse call relations.
        for callee in &callees {
            self.back_edges
                .entry(callee.clone())
                .and_modify(|caller| _ = caller.set.insert(fn_item.clone()))
                .or_insert_with(|| Nodes {
                    set: IndexSet::from([fn_item.clone()]),
                });
        }

        // Add direct callees nodes. (caller -> callees)
        self.edges.insert(fn_item, nodes);

        for callee in callees {
            // Recurse.
            self.reach_in_depth(callee);
        }
    }

    /// Sort keys and values by fn names.
    fn sort(&mut self) {
        self.edges.sort_by(|f1, _, f2, _| f1.cmp(f2));
        self.back_edges.sort_by(|f1, _, f2, _| f1.cmp(f2));
        for node in self.edges.values_mut().chain(self.back_edges.values_mut()) {
            node.set.sort_by(|f1, f2| f1.cmp(f2));
        }
    }

    fn print(&self) {
        dbg!(self);
    }
}

#[derive(Debug, Default)]
struct Nodes {
    set: IndexSet<FnItem>,
}

impl MirVisitor for Nodes {
    fn visit_ty(&mut self, ty: &Ty, _: Location) {
        // We don't need GenericArgs, focusing on the function item.
        if let TyKind::RigidTy(RigidTy::FnDef(fn_def, _)) = ty.kind() {
            self.set.insert(fn_def.into());
        }
        self.super_ty(ty);
    }
}

/// A FnDef simplified on Debug trait and `{:?}` printing.
#[derive(Clone, PartialEq, Eq, Hash)]
struct FnItem {
    def: FnDef,
    name: Rc<str>,
}
impl fmt::Debug for FnItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}
impl PartialOrd for FnItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FnItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
impl From<FnDef> for FnItem {
    fn from(fn_def: FnDef) -> Self {
        Self::new(fn_def)
    }
}
impl FnItem {
    fn new(fn_def: FnDef) -> Self {
        FnItem {
            def: fn_def,
            name: fn_def.name().into(),
        }
    }
}
