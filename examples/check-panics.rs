//! Check panic spots.
//!
//! `cargo run --example check-panics -- examples/vec-push/vec-push.rs`
//!
//! ```rust,ignore
//! self.backtrace_path_starting_by_name(PANIC) = [
//!     [
//!         "core::panicking::panic_nounwind",
//!         "core::ub_checks::maybe_is_nonoverlapping::runtime",
//!         "std::ptr::copy_nonoverlapping::precondition_check",
//!         "std::alloc::Allocator::grow",
//!         "alloc::raw_vec::RawVecInner::<A>::finish_grow",
//!         "alloc::raw_vec::RawVecInner::<A>::grow_amortized",
//!         "alloc::raw_vec::RawVec::<T, A>::grow_one",
//!         "std::vec::Vec::<T, A>::push_mut",
//!         "std::vec::Vec::<T, A>::push",
//!         "main",
//!     ],
//! ]
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
use std::{fmt, ops::ControlFlow, rc::Rc, sync::LazyLock};

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

    /// Search FnItem/DefId.
    fn get_fn_item(&self, fn_name: &str) -> Option<&FnItem> {
        self.edges.keys().find(|f| f.is(fn_name))
    }

    /// From a nested callee to the top-level crate fn item (caller).
    fn backtrace_path(&self, fn_item: FnItem) -> CallPaths {
        let mut path = Vec::new();
        let mut v_path = Vec::new();

        path.push(fn_item.clone());
        self.add_back_path(&fn_item, &mut path, &mut v_path);

        v_path
    }

    fn add_back_path(&self, f: &FnItem, path: &mut Vec<FnItem>, v_path: &mut CallPaths) {
        if let Some(callers) = self.back_edges.get(f) {
            for caller in &callers.set {
                path.push(caller.clone());
                // Recurse.
                self.add_back_path(caller, path, v_path);
            }
            return;
        }
        // The outmost caller doesn't have any caller, reaching the end.
        v_path.push(path.clone());
        path.pop();
    }

    fn backtrace_path_starting_by_name(&self, fn_name: &str) -> CallPaths {
        self.get_fn_item(fn_name)
            .cloned()
            .map(|f| self.backtrace_path(f))
            .unwrap_or_default()
    }

    fn call_path(&self, start: &FnItem, stop: &FnItem) -> CallPaths {
        let mut path = Vec::new();
        let mut v_path = Vec::new();

        path.push(start.clone());
        self.add_call_path(start, stop, &mut path, &mut v_path);

        v_path
    }

    fn add_call_path(
        &self,
        start: &FnItem,
        stop: &FnItem,
        path: &mut Vec<FnItem>,
        v_path: &mut CallPaths,
    ) {
        if let Some(callees) = self.edges.get(start) {
            for callee in &callees.set {
                path.push(callee.clone());
                if callee == stop {
                    v_path.push(path.clone());
                } else {
                    self.add_call_path(callee, stop, path, v_path);
                    path.pop();
                }
            }
        }
    }

    fn print(&self) {
        const PANIC: &str = "core::panicking::panic_nounwind";
        const PANIC_FMT: &str = "core::panicking::panic_nounwind_fmt";

        let midpoint = "std::alloc::Allocator::grow";

        dbg!(
            self,
            self.backtrace_path_starting_by_name(PANIC),
            self.backtrace_path_starting_by_name(PANIC_FMT),
            // We can cut through any call to trace back.
            self.backtrace_path_starting_by_name(midpoint),
        );

        let main_to_panic = self
            .call_path(
                self.get_fn_item("main").unwrap(),
                self.get_fn_item(PANIC).unwrap(),
            )
            .iter()
            .map(|p| p.iter().map(|f| f.print()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let midpoint_to_panic = self.call_path(
            self.get_fn_item(midpoint).unwrap(),
            self.get_fn_item(PANIC).unwrap(),
        );
        dbg!(main_to_panic, midpoint_to_panic);
    }
}

type CallPaths = Vec<Vec<FnItem>>;

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

    fn is(&self, name: &str) -> bool {
        *self.name == *name
    }

    /// An alternative debug string containing the name and span.
    fn print(&self) -> String {
        // .diagnostic() contain absolute sysroot path, thus strip to shorten it
        static PREFIX: LazyLock<Box<str>> = LazyLock::new(|| {
            let output = std::process::Command::new("rustc")
                .arg("--print=sysroot")
                .output()
                .unwrap();
            assert!(output.status.success());
            let sysroot = std::str::from_utf8(&output.stdout).unwrap().trim();
            format!("{sysroot}/lib/rustlib/src/rust/library/").into()
        });

        let span = self.def.span().diagnostic();
        format!("{} ({})", self.name, span.trim_start_matches(&**PREFIX))
    }
}
