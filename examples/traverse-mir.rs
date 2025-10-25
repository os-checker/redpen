//! Collect direct called funtions.
//!
//! `cargo run --example traverse-mir -- examples/vec-push/vec-push.rs`
//!
//! ```rust,ignore
//! collector = CalleeCollector {
//!    v_callee: [
//!        Callee {
//!            fn_def: "std::vec::Vec::<T>::new",
//!            generics: [
//!                "Int(I32)",
//!            ],
//!        },
//!        Callee {
//!            fn_def: "std::vec::Vec::<T, A>::push",
//!            generics: [
//!                "Int(I32)",
//!                "Adt(AdtDef(DefId { id: 4, name: \"std::alloc::Global\" }), GenericArgs([]))",
//!            ],
//!        },
//!    ],
//! }
//! ```
#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_public::{
    CrateDef,
    mir::{MirVisitor, visit::Location},
    ty::{FnDef, GenericArgs, RigidTy, Ty, TyKind},
};
use std::{fmt, ops::ControlFlow};

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, analysis);
}

fn analysis() -> ControlFlow<(), ()> {
    let mut collector = CalleeCollector::default();
    let local_crate = rustc_public::local_crate();

    for f in local_crate.fn_defs() {
        if let Some(body) = f.body() {
            collector.visit_body(&body);
        }
    }

    dbg!(&collector);
    ControlFlow::Break(())
}

struct Callee {
    fn_def: FnDef,
    generics: GenericArgs,
}

impl fmt::Debug for Callee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.generics.0.iter();
        let args: Vec<_> = iter
            .filter_map(|arg| {
                if let TyKind::RigidTy(ty) = arg.ty()?.kind() {
                    Some(format!("{ty:?}"))
                } else {
                    None
                }
            })
            .collect();
        f.debug_struct("Callee")
            .field("fn_def", &self.fn_def.name())
            .field("generics", &args)
            .finish()
    }
}

#[derive(Debug, Default)]
struct CalleeCollector {
    v_callee: Vec<Callee>,
}

impl MirVisitor for CalleeCollector {
    fn visit_ty(&mut self, ty: &Ty, _: Location) {
        if let TyKind::RigidTy(RigidTy::FnDef(fn_def, generics)) = ty.kind() {
            self.v_callee.push(Callee { fn_def, generics });
        }
        self.super_ty(ty);
    }
}
