use crate::{detect::Detect, diagnostics::PanicSpots, fn_item::FnItem};
use indexmap::{IndexMap, IndexSet};
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    mir::{Body, MirVisitor, Operand, visit::Location},
    rustc_internal::internal,
    ty::{FnDef, RigidTy, Span, Ty, TyKind},
};

#[derive(Debug, Default)]
pub struct CallGraph {
    edges: IndexMap<FnItem, Nodes>,
    back_edges: IndexMap<FnItem, Nodes>,
}

impl CallGraph {
    /// Reach crate function items as entries.
    pub fn reach_in_depth(&mut self, fn_item: FnItem) {
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
    pub fn sort(&mut self) {
        self.edges.sort_by(|f1, _, f2, _| f1.cmp(f2));
        self.back_edges.sort_by(|f1, _, f2, _| f1.cmp(f2));
        for node in self.edges.values_mut().chain(self.back_edges.values_mut()) {
            node.set.sort_by(|f1, f2| f1.cmp(f2));
        }
    }

    /// Search FnItem/DefId.
    pub fn get_fn_item(&self, fn_name: &str) -> Option<&FnItem> {
        self.edges.keys().find(|f| f.is(fn_name))
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

    fn reachable(&self, start: &FnItem, stop: &FnItem) -> bool {
        if let Some(callees) = self.edges.get(start) {
            for callee in &callees.set {
                if callee == stop || self.reachable(callee, stop) {
                    return true;
                }
            }
        }
        false
    }

    pub fn analyze(&self, detect: &Detect, tcx: TyCtxt) -> PanicSpots {
        // dbg!(self);
        let verbose = true;
        let mut spots = PanicSpots::default();

        let mut v_panic = Vec::new();
        detect.with_panic_item(|f| v_panic.push(f.clone()));
        if v_panic.is_empty() {
            return spots;
        };

        for entry in detect.entries() {
            for panic in &v_panic {
                if !verbose && self.reachable(entry, panic) {
                    println!("{entry:?} reaches panic");
                } else {
                    let caller = entry.def;
                    let body = caller.body().unwrap();
                    let span = body.span;

                    let path = self.call_path(entry, panic);
                    let mut local_spots = LocalPanicSpot::new(&path, &body, tcx);
                    local_spots.visit_body(&body);
                    spots.add(caller, span, local_spots.panic_spots());
                }
            }
        }
        spots
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

pub fn contains_span(tcx: TyCtxt, large: Span, small: Span) -> bool {
    let large = internal(tcx, large);
    let small = internal(tcx, small);
    large.contains(small)
}

struct LocalPanicSpot<'tcx, 'body> {
    tcx: TyCtxt<'tcx>,
    caller_body: &'body Body,
    fn_may_panic: IndexSet<FnDef>,
    panic_spots: IndexMap<FnDef, Vec<Span>>,
}

impl<'tcx, 'body> LocalPanicSpot<'tcx, 'body> {
    fn new(path: &CallPaths, body: &'body Body, tcx: TyCtxt<'tcx>) -> Self {
        LocalPanicSpot {
            tcx,
            caller_body: body,
            fn_may_panic: path.iter().flatten().map(|a| a.def).collect(),
            panic_spots: Default::default(),
        }
    }

    fn contains(&self, span: Span) -> bool {
        contains_span(self.tcx, self.caller_body.span, span)
    }

    fn panic_spots(self) -> IndexSet<Span> {
        self.panic_spots.into_values().flatten().collect()
    }

    fn check_panic_spot(&mut self, ty: &Ty, span: Span) {
        if let Some((fn_def, _)) = ty.kind().fn_def()
            && self.fn_may_panic.contains(&fn_def)
            && self.contains(span)
        {
            self.panic_spots
                .entry(fn_def)
                .and_modify(|v| v.push(span))
                .or_insert_with(|| vec![span]);
        }
    }
}

impl MirVisitor for LocalPanicSpot<'_, '_> {
    // fn visit_span(&mut self, span: &Span) {
    //     if self.contains(*span) {
    //         dbg!(span);
    //     }
    // }

    fn visit_operand(&mut self, operand: &Operand, location: Location) {
        if let Ok(ty) = operand.ty(self.caller_body.locals()) {
            let span = location.span();
            self.check_panic_spot(&ty, span);
        }
        self.super_operand(operand, location);
    }
}
