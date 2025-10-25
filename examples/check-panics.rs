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
use std::{fmt, ops::ControlFlow};

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let mut reachability = Reachability::default();
    let local_crate = rustc_public::local_crate();

    for f in local_crate.fn_defs() {
        reachability.reach_in_depth(FnItem(f));
    }

    reachability.print();
    ControlFlow::Break(())
}

#[derive(Debug, Default)]
struct Reachability {
    map: IndexMap<FnItem, Visitor>,
}

impl Reachability {
    /// Reach crate function items as entries.
    fn reach_in_depth(&mut self, fn_item: FnItem) {
        if self.map.contains_key(&fn_item) {
            // The fn item has been reached before.
            return;
        }

        let mut visitor = Visitor::default();
        if let Some(body) = fn_item.0.body() {
            visitor.visit_body(&body);
        }

        // Add direct callees on callees.
        let callees: IndexSet<_> = visitor.callees.iter().copied().collect();

        self.map.insert(fn_item, visitor);

        for callee in callees {
            // Recurse.
            self.reach_in_depth(callee);
        }
    }

    fn print(&self) {
        dbg!(self);
    }
}

#[derive(Debug, Default)]
struct Visitor {
    callees: IndexSet<FnItem>,
}

impl MirVisitor for Visitor {
    fn visit_ty(&mut self, ty: &Ty, _: Location) {
        // We don't need GenericArgs, focusing on the function item.
        if let TyKind::RigidTy(RigidTy::FnDef(fn_def, _)) = ty.kind() {
            self.callees.insert(FnItem(fn_def));
        }
        self.super_ty(ty);
    }
}

/// A FnDef simplified on Debug trait and `{:?}` printing.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct FnItem(FnDef);
impl fmt::Debug for FnItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.name().fmt(f)
    }
}
