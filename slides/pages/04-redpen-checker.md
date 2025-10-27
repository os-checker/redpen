# Redpen Checker

<div class="columns-2">
<Toc mode="onlyCurrentTree" />
</div>

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

## 调用关系图

<CodeblockSmallSized>

<TwoColumns>

<template #left>

```rust
edges: {
    "main": Nodes {
        "std::vec::Vec::<T, A>::push",
        "std::vec::Vec::<T>::new",
    },
    "std::vec::Vec::<T, A>::push": Nodes {
        "std::vec::Vec::<T, A>::push_mut",
    },
    "std::vec::Vec::<T, A>::push_mut": Nodes {
        "alloc::raw_vec::RawVec::<T, A>::grow_one",
    },
    ...
}
```

[reachability.txt](https://github.com/os-checker/redpen/blob/main/examples/vec-push/reachability.txt)

</template>

<template #right>

```rust
back_edges: {
    "std::vec::Vec::<T, A>::push": Nodes {
        "main",
    },
    "std::vec::Vec::<T>::new": Nodes {
        "main",
    },
    "std::vec::Vec::<T, A>::push_mut": Nodes {
        "std::vec::Vec::<T, A>::push",
    },
    "core::panicking::panic_nounwind": Nodes {
        "core::ub_checks::maybe_is_nonoverlapping::runtime",
    },
    "core::panicking::panic_nounwind_fmt": Nodes {
        "core::num::<impl usize>::unchecked_mul::precondition_check",
        "std::alloc::Layout::from_size_align_unchecked::precondition_check",
        "std::hint::assert_unchecked::precondition_check",
        "std::ptr::copy_nonoverlapping::precondition_check",
    },
    ...
}
```

</template>

</TwoColumns>


</CodeblockSmallSized>

---

## CallGraph::call_path

<CodeblockSmallSized>
<TwoColumns>

<template #left>

```rust
fn call_path(
    &self, start: &FnItem, stop: &FnItem
) -> CallPaths {
    let mut path = Vec::new();
    let mut v_path = Vec::new();

    path.push(start.clone());

    self.add_call_path(
      start, stop, &mut path, &mut v_path
    );

    v_path
}

type CallPaths = Vec<Vec<FnItem>>;
```

</template>

<template #right>

```rust
fn add_call_path(
    &self, start: &FnItem, stop: &FnItem,
    path: &mut Vec<FnItem>, v_path: &mut CallPaths
) {
    if let Some(callees) = self.edges.get(start) {
        for callee in &callees.set {
            path.push(callee.clone());
            if callee == stop {
                v_path.push(path.clone());
            } else {
                // Recurse.
                self.add_call_path(
                  callee, stop, path, v_path
                );
                path.pop();
            }
        }
    }
}
```

</template>

</TwoColumns>
</CodeblockSmallSized>

---

## 调用路径

从 `main` 函数到 `panic_nounwind` 函数：

<CodeblockSmallSized>

```rust
main_to_panic = [
  [
    "main",
    "std::vec::Vec::<T, A>::push",
    "std::vec::Vec::<T, A>::push_mut",
    "alloc::raw_vec::RawVec::<T, A>::grow_one",
    "alloc::raw_vec::RawVecInner::<A>::grow_amortized",
    "alloc::raw_vec::RawVecInner::<A>::finish_grow",
    "std::alloc::Allocator::grow",
    "std::ptr::copy_nonoverlapping::precondition_check",
    "core::ub_checks::maybe_is_nonoverlapping::runtime",
    "core::panicking::panic_nounwind",
  ],
]
```

```rust
/// Search FnItem/DefId.
fn get_fn_item(&self, fn_name: &str) -> Option<&FnItem> {
    self.edges.keys().find(|f| f.is(fn_name))
}
```

</CodeblockSmallSized>

---

## CallGraph::backtrace_path

<CodeblockSmallSized>

```rust
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
```

</CodeblockSmallSized>

[reachability.txt](https://github.com/os-checker/redpen/blob/128304aff6f68c7f4f92822985f49f9568c31c2a/examples/vec-push/reachability.txt#L299-L420)

<style> p { margin: 0; } </style>

---

## Detect 出入口

<CodeblockSmallSized>
<TwoColumns>

<template #left>

```rust
pub struct Detect {
    panic_nounwind: Option<FnItem>,
    panic_nounwind_fmt: Option<FnItem>,
    begin_panic: Option<FnItem>,
    panic_fmt: Option<FnItem>,
    entries: Vec<FnItem>, // 可达性入口
}

pub fn is_panic_fn(&self, fn_def: &FnDef) -> bool {
    let mut b = false;
    self.with_panic_item(|f| b |= f.def == *fn_def);
    b
}
```

```rust
// Vec::push 内部达到不同的 panic 函数
core::panicking::panic_nounwind -> main
core::panicking::panic_nounwind_fmt -> main

// 不同 edition 使用不同的 panic 函数
std::rt::begin_panic -> main
std::rt::panic_fmt -> main
```

</template>

<template #right>

```rust
pub fn new(
  call_graph: &CallGraph, entries: Vec<FnItem>
) -> Self {
    const PANIC_NOUNWIND: &str =
      "core::panicking::panic_nounwind";
    const PANIC_NOUNWIND_FMT: &str =
      "core::panicking::panic_nounwind_fmt";

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
```

</template>

</TwoColumns>
</CodeblockSmallSized>

---

## panic 检查

<TwoColumns left="40%" right="60%">

<template #left>

报告函数可能 panic 的直接调用：

```rust
pub fn panic() {
    panic!("💥")
}

struct S {}
impl S {
    pub fn caller(&self) {
        panic();
        panic!("Second panic.")
    }
}
```

</template>

<template #right>
<PanicSpots />
</template>

</TwoColumns>

---

<CodeblockSmallSized>
<TwoColumns>

<template #left>

## LocalPanicSpot

<div class="h-full flex items-center">
<div>

```rust
struct LocalPanicSpot<'tcx, 'body, 'detect> {
    // 用于操作 Span
    tcx: TyCtxt<'tcx>,
    // 函数体（基本块、局部变量）
    caller_body: &'body Body,
    // panic 函数
    detect: &'detect Detect,
    // panic 路径上的函数
    fn_may_panic: IndexSet<FnDef>,
    // panic 路径的函数在函数体内部的调用位置
    panic_spots: IndexMap<FnDef, Vec<Span>>,
}

use rustc_public::rustc_internal::internal;
fn contains(&self, span: Span) -> bool {
    let large = self.caller_body.span;
    let large = internal(self.tcx, large);
    let small = internal(self.tcx, span);
    large.contains(small)
}
```

<div class="text-center">
搜集函数内部位于 panic 路径上的 Span 
</div>

</div>
</div>
</template>

<template #right>

```rust
impl MirVisitor for LocalPanicSpot<'_, '_, '_> {
    fn visit_operand(&mut self, operand: &Operand, location: Location) {
        if let Ok(ty) = operand.ty(self.caller_body.locals()) {
            let span = location.span();
            self.check_panic_spot(&ty, span);
        }
        self.super_operand(operand, location);
    }
}
```

```rust
fn check_panic_spot(&mut self, ty: &Ty, span: Span) {
    if let Some((fn_def, _)) = ty.kind().fn_def()
        && self.fn_may_panic.contains(&fn_def)
        && self.contains(span)
    {
        self.add(fn_def, span);
    }
}

fn add(&mut self, fn_def: FnDef, span: Span) {
    self.panic_spots
        .entry(fn_def)
        .and_modify(|v| v.push(span))
        .or_insert_with(|| vec![span]);
}
```

</template>

</TwoColumns>
</CodeblockSmallSized>


---

## CallGraph::analyze

<CodeblockSmallSized>
<TwoColumns left="60%" right="40%">

<template #left>

```rust
fn analyze(&self, detect: &Detect, tcx: TyCtxt) -> PanicSpots {
    let mut spots = PanicSpots::default();

    let mut v_panic = Vec::new();
    detect.with_panic_item(|f| v_panic.push(f.clone()));
    if v_panic.is_empty() {
        return spots; // 该函数不含 panic 路径
    };

    for entry in detect.entries() {
        for panic in &v_panic {
            let caller = entry.def;
            let body = caller.body().unwrap();
            let span = body.span;

            let path = self.call_path(entry, panic); // panic 路径
            let mut local_spots = LocalPanicSpot::new(
                &path, &body, detect, tcx);
            local_spots.visit_body(&body); // 遍历函数体
            spots.add(caller, span, local_spots.panic_spots());
        }
    }
    spots
}
```

</template>

<template #right>

```rust
pub struct PanicSpots {
    // 携带 panic 路径的函数及相关的调用位置
    map: IndexMap<FnDef, Spots>,
}

impl PanicSpots {
    pub fn add(&mut self,
        caller: FnDef, span_caller: PubSpan,
        span_callee: IndexSet<PubSpan>
    ) {
        ...
    }
}

struct Spots {
    // 函数主体的位置
    caller: PubSpan,
    // 传递 panic 调用的位置
    calls: IndexSet<PubSpan>,
}
```

</template>

</TwoColumns>
</CodeblockSmallSized>

---

## 发出诊断


<CodeblockSmallSized>
<TwoColumns>

<template #left>

```rust
let span_func = span(self.spots.caller, self.src.tcx);
let source_map = &self.src.src_map;

let source = source_map
  .span_to_snippet(span_func).unwrap();

let pos_func = span_func.lo();
let loc = source_map.lookup_char_pos(pos_func);
let file_path = loc.file.name
  .prefer_remapped_unconditionally().to_string();

let offset = |sp: PubSpan| { // 计算相对于 caller 的位置
    let span = span(sp, tcx);
    let call_span_lo = span.lo() - pos_func;
    let call_span_hi = span.hi() - pos_func;
    call_span_lo.0 as usize..call_span_hi.0 as usize
};
```

<PanicSpotsDemo />

</template>

<template #right>

```rust
let annot_caller = AnnotationKind::Context
    .span(offset(self.f))
    .label("For this function.");

let annot_call = |sp: PubSpan| {
    AnnotationKind::Primary
        .span(offset(sp))
        .label("This may panic!")
};

let diag = Level::ERROR
    .primary_title("A possible panic spot is found.")
    .element(
        Snippet::source(source)
            .path(file_path)
            .line_start(loc.line)
            .annotation(annot_caller)
            .annotations(
              self.spots.calls
                .iter().copied().map(annot_call))
    );

eprintln!("{}", renderer.render(&[diag]));
```

</template>

</TwoColumns>
</CodeblockSmallSized>

---

## `#[redpen::silence_panic]`

<CodeblockSmallSized>
<TwoColumns>

<template #left>

```rust {1,5-14}
for f in local_crate.fn_defs() {
    let fn_item = FnItem::new(f);
    call_graph.reach_in_depth(fn_item.clone());

    // When a top-level function is tagged, 
    // don't treat it as an entry item to report.
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
```


</template>

<template #right>

```rust {1,2,9,10}
#![feature(register_tool)]
#![register_tool(redpen)]

fn main() {
    let mut v = vec![0];
    v.push(1);
}

#[redpen::silence_panic]
pub fn dont_report() {
    let mut v = vec![0];
    v.push(1);
}
```

</template>

</TwoColumns>
</CodeblockSmallSized>


---

## 快照测试

<CodeblockSmallSized>
<TwoColumns>

<template #left>

```rust
#[test] // tests/check-panics.rs
fn run_ui_tests() {
    let bless = env::var("BLESS").is_ok_and(
        |x| !x.trim().is_empty());
    let mut config = compiletest::Config {
        bless, // 是否更新快照
        mode: compiletest::common::Mode::Ui,
        ..Default::default()
    };

    // 增加额外的编译器参数
    config.target_rustcflags = Some("--crate-type=lib \
      -Zcrate-attr=feature(register_tool) \
      -Zcrate-attr=register_tool(redpen)".into()
    );

    config.src_base = "tests/ui".into();
    config.build_base = PROFILE_PATH.join("test/ui");
    config.rustc_path = PROFILE_PATH.join("redpen");
    // Populate rustcflags with dependencies on the path
    config.link_deps();

    compiletest::run_tests(&config);
}
```

</template>

<template #right>

```bash
$ cargo test # 对比诊断：出现差异则测试失败
$ BLESS=1 cargo test # 更新快照
```

```toml
[dev-dependencies]
compiletest_rs = "0.11.2"
```

```bash
 tests
  ui
│ │ 󱘗 1-direct.rs
│ │ * 1-direct.stderr
│ │ 󱘗 1-indirect.rs
│ │ * 1-indirect.stderr
│ │ 󱘗 2-method.rs
│ │ * 2-method.stderr
│ │ 󱘗 no-panic.rs
│ └ 󱘗 silence-panic.rs
└ 󱘗 check-panics.rs
```

</template>

</TwoColumns>
</CodeblockSmallSized>

---

## cargo-redpen

<CodeblockSmallSized>
<TwoColumns>

<template #left>

```bash
  src
    bin
      redpen
    │ │ 󱘗 call_graph.rs
    │ │ 󱘗 detect.rs
    │ │ 󱘗 diagnostics.rs
    │ │ 󱘗 fn_item.rs
    │ └ 󱘗 main.rs 👈
    └ 󱘗 cargo-redpen.rs 👈
  tests
   Cargo.toml
   rust-toolchain.toml
```

```bash
# Install redpen and cargo-redpen.
cd redpen
cargo install --path . --locked

# Run checker.
cd tests/vec-push
cargo redpen

# For your project that doesn't use the same toolchain:
cargo +nightly-2025-10-23 redpen
```

</template>

<template #right>

```rust
fn main() { // src/bin/cargo-redpen.rs
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 2 && args[1].as_str() == "-vV" {
        run("rustc", &["-vV".to_owned()], &[]);
    } else if std::env::var("WRAPPER").as_deref() == Ok("1") {
        run("redpen", &args[1..], &[]);
    } else {
        run("cargo", &args, // cargo build args...
          &[("RUSTC", "cargo-redpen"), ("WRAPPER", "1")]);
    }
}

fn run(cmd: &str, args: &[String], vars: &[(&str, &str)]) {
    let status = Command::new(cmd).args(args)
                  .envs(vars).spawn().wait();
    if !status.success() {
        std::process::abort()
    }
}
```

</template>

</TwoColumns>
</CodeblockSmallSized>


