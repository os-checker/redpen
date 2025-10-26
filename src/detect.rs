use crate::{call_graph::CallGraph, fn_item::FnItem};

pub struct Detect {
    panic_nounwind: Option<FnItem>,
    panic_nounwind_fmt: Option<FnItem>,
    begin_panic: Option<FnItem>,
    panic_fmt: Option<FnItem>,
    entries: Vec<FnItem>,
}

impl Detect {
    pub fn new(call_graph: &CallGraph, entries: Vec<FnItem>) -> Self {
        const PANIC_NOUNWIND: &str = "core::panicking::panic_nounwind";
        const PANIC_NOUNWIND_FMT: &str = "core::panicking::panic_nounwind_fmt";
        const BEGIN_PANIC: &str = "std::rt::begin_panic";
        const PANIC_FMT: &str = "std::rt::panic_fmt";

        let get = |name: &str| call_graph.get_fn_item(name).cloned();

        Detect {
            panic_nounwind: get(PANIC_NOUNWIND),
            panic_nounwind_fmt: get(PANIC_NOUNWIND_FMT),
            begin_panic: get(BEGIN_PANIC),
            panic_fmt: get(PANIC_FMT),
            entries,
        }
    }

    pub fn entries(&self) -> &[FnItem] {
        &self.entries
    }

    pub fn with_panic_item(&self, mut f: impl FnMut(&FnItem)) {
        let panic_items = [
            &self.panic_nounwind,
            &self.panic_nounwind_fmt,
            &self.begin_panic,
            &self.panic_fmt,
        ];
        for panic in panic_items.iter().filter_map(|&f| f.as_ref()) {
            f(panic);
        }
    }

    // pub fn is_panic_fn(&self, fn_def: &FnDef) -> bool {
    //     let mut b = false;
    //     self.with_panic_item(|f| b |= f.def == *fn_def);
    //     b
    // }
}
