//! Collect nested called funtions.
//!
//! `cargo run --example traverse-mir-and-instances -- examples/vec-push/vec-push.rs`
//!
//! ```rust,ignore
//! callees.len() = 85
//! callees = [
//!     "std::vec::Vec::<T>::new",
//!     "std::vec::Vec::<T, A>::push",
//!     "std::vec::Vec::<T, A>::push_mut",
//!     "alloc::raw_vec::RawVec::<T, A>::grow_one",
//!     "alloc::raw_vec::RawVecInner::<A>::grow_amortized",
//!     "std::intrinsics::cold_path",
//!     "std::cmp::Ord::max",
//!     "std::cmp::PartialOrd::lt",
//!     "std::cmp::Ord::max",
//!     "std::cmp::PartialOrd::lt",
//!     "alloc::raw_vec::RawVecInner::<A>::finish_grow",
//!     "std::alloc::Layout::repeat",
//!     "std::alloc::Layout::from_size_align_unchecked::precondition_check",
//!     "std::alloc::Layout::is_size_align_valid",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::intrinsics::cold_path",
//!     "std::alloc::Allocator::allocate",
//!     "std::alloc::Global::alloc_impl",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "alloc::alloc::__rust_alloc_zeroed",
//!     "alloc::alloc::__rust_alloc",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "alloc::raw_vec::RawVecInner::<A>::current_memory",
//!     "core::num::<impl usize>::unchecked_mul::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::alloc::Layout::from_size_align_unchecked::precondition_check",
//!     "std::alloc::Layout::is_size_align_valid",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::hint::assert_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::alloc::Allocator::grow",
//!     "std::alloc::Global::grow_impl",
//!     "std::alloc::Global::alloc_impl",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "alloc::alloc::__rust_alloc_zeroed",
//!     "alloc::alloc::__rust_alloc",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::alloc::Global::alloc_impl",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "alloc::alloc::__rust_no_alloc_shim_is_unstable_v2",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "alloc::alloc::__rust_alloc_zeroed",
//!     "alloc::alloc::__rust_alloc",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::hint::assert_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "alloc::alloc::__rust_realloc",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::ptr::write_bytes::precondition_check",
//!     "std::intrinsics::ctpop",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::rt::panic_fmt",
//!     "std::intrinsics::write_bytes",
//!     "std::ptr::NonNull::<T>::new_unchecked::precondition_check",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::ptr::copy_nonoverlapping::precondition_check",
//!     "std::intrinsics::ctpop",
//!     "std::intrinsics::ctpop",
//!     "std::intrinsics::ctpop",
//!     "core::ub_checks::maybe_is_nonoverlapping::runtime",
//!     "std::intrinsics::cold_path",
//!     "core::panicking::panic_nounwind",
//!     "core::panicking::panic_nounwind_fmt",
//!     "std::rt::panic_fmt",
//!     "std::rt::panic_fmt",
//!     "alloc::alloc::__rust_dealloc",
//!     "alloc::alloc::__rust_realloc",
//!     "alloc::raw_vec::handle_error",
//! ]
//! ```
#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_public::{
    CrateDef,
    mir::{MirVisitor, mono::Instance, visit::Location},
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

    collector.print();
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

impl CalleeCollector {
    fn print(&self) {
        let callees: Vec<_> = self.v_callee.iter().map(|f| f.fn_def.name()).collect();
        dbg!(callees.len(), callees);
    }
}

impl MirVisitor for CalleeCollector {
    fn visit_ty(&mut self, ty: &Ty, _: Location) {
        if let TyKind::RigidTy(RigidTy::FnDef(fn_def, generics)) = ty.kind() {
            let opt_instance = Instance::resolve(fn_def, &generics);

            self.v_callee.push(Callee { fn_def, generics });

            if let Ok(instance) = opt_instance
                && let Some(body) = instance.body()
            {
                self.visit_body(&body);
            }
        }
        self.super_ty(ty);
    }
}
