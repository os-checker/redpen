# Redpen Checker

<!-- <div class="columns-2"> -->
<Toc mode="onlyCurrentTree" />
<!-- </div> -->

<BackToTOC />

---

## Check Panics 目标

找出所有到达的 `panic` 调用：

* 可达性分析（基于 MirVisitor 示例）
* 打印调用发生的路径
* 打印调用发生的位置


---

<TwoColumns>

<template #left>

## CallGraph::reach_in_depth

<div class="h-4" />
<CodeblockSmallSized>

```rust
struct CallGraph {
  edges: IndexMap<FnItem, Nodes>,
  back_edges: IndexMap<FnItem, Nodes>,
}

#[derive(Debug, Default)]
struct Nodes {
  set: IndexSet<FnItem>,
}

impl MirVisitor for Nodes {
  fn visit_ty(&mut self, ty: &Ty, _: Location) {
    if let RigidTy(FnDef(fn_def, _)) = ty.kind() {
       self.set.insert(fn_def.into());
    }
    self.super_ty(ty);
  }
}
```

* [redpen/examples/check-panics.rs](https://github.com/os-checker/redpen/blob/128304aff6f68c7f4f92822985f49f9568c31c2a/examples/check-panics.rs#L63)
* [MirVisitor::super_body](https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_public/mir/visit.rs.html#387)

</CodeblockSmallSized>
</template>

<template #right>

<CodeblockSmallSized font-size="10px" row-gap="5pt">

```rust
fn reach_in_depth(&mut self, fn_item: FnItem) {
    if self.edges.contains_key(&fn_item) { return; }

    let mut nodes = Nodes::default();
    if let Some(body) = fn_item.def.body() {
        nodes.visit_body(&body); // Visit subitems and ty.
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
        self.reach_in_depth(callee); // Recurse.
    }
}
```

</CodeblockSmallSized>
</template>

</TwoColumns>


---

References:

* [Rust Dev Guide: Diagnostics](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
* [Clippy Development](https://doc.rust-lang.org/clippy/development/index.html)
* [`rustc_middle::lint::lint_level`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/lint/fn.lint_level.html) to emit lints

