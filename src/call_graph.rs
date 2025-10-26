use crate::{detect::Detect, fn_item::FnItem};
use indexmap::{IndexMap, IndexSet};
use rustc_public::{
    mir::{MirVisitor, Operand, TerminatorKind, visit::Location},
    ty::{RigidTy, Ty, TyKind},
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

    pub fn diagnostic(&self, detect: &Detect) {
        // dbg!(self);
        let verbose = true;

        let mut v_panic = Vec::new();
        detect.with_panic_item(|f| v_panic.push(f.clone()));
        if v_panic.is_empty() {
            return;
        };

        for entry in detect.entries() {
            for panic in &v_panic {
                if !verbose && self.reachable(entry, panic) {
                    println!("{entry:?} reaches panic");
                } else {
                    let call_path = self.call_path(entry, panic);
                    for path in &call_path {
                        dbg!(path.iter().map(|f| f.print()).collect::<Vec<_>>());

                        let caller = entry.def.body().unwrap();

                        match path.len() {
                            // panic happens in the caller
                            2 => {
                                // dbg!(&caller);
                                for bb in &caller.blocks {
                                    // Only handle `begin_panic(arg)` for now:
                                    // the begin_panic arg points to a local span.
                                    if let TerminatorKind::Call { func, args, .. } =
                                        &bb.terminator.kind
                                        && let Operand::Constant(val) = func
                                        && let Some((fn_def, _)) = val.ty().kind().fn_def()
                                        && detect.is_panic_fn(&fn_def)
                                        && let Some(Operand::Constant(operand)) = args.first()
                                    {
                                        _ = dbg!(operand.span)
                                    }
                                }
                            }
                            // Panic happens in the callee, so fetch the local call span.
                            _ => {
                                let call = &path[1];
                                for bb in &caller.blocks {
                                    if let TerminatorKind::Call { func, .. } = &bb.terminator.kind
                                        && let Operand::Constant(val) = func
                                        && let Some((fn_def, _)) = val.ty().kind().fn_def()
                                        && fn_def == call.def
                                    {
                                        dbg!(bb.terminator.span);
                                    }
                                }
                            }
                        };
                    }
                }
            }
        }
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
