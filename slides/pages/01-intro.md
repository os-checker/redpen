## 固定工具链

<div class="DivWide">

1. 添加 `rust-toolchain.toml` 文件：

```toml
# 设置工具链和组件
[toolchain]
channel = "nightly-2025-10-23"
components = ["llvm-tools", "rustc-dev", "rust-src", "rustfmt", "clippy"]
```

2. `Cargo.toml` 让 Rust-Analyzer 访问编译器内部的 API：

```toml
# 我们需要通过 #![feature(rustc_private)] 访问编译器 API
[package.metadata.rust-analyzer]
rustc_private = true
```

</div>


---

## 编译器 API 入口

```rust
#![feature(rustc_private)]

extern crate rustc_public;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, || ControlFlow::<(), ()>::Break(()));
}
```

<div class="CodeblockTitle text-red-500">

[rustc_public::run!](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/macro.run.html)

</div>

* `#![feature(rustc_private)]` 允许源代码访问编译器 API
* `extern crate` 引入编译器模块；见 [nightly-rustc](https://doc.rust-lang.org/nightly/nightly-rustc) API 文档
* `rustc_args` 是从命令行转发给编译器的参数
* `ControlFlow::<(), ()>::Break(())` 让编译器在分析代码之后不生成二进制文件

---

<TwoColumns left="70%" right="30%">

<template #left>

  <div>

```rust
error[E0433]: failed to resolve: use of unresolved module
              or unlinked crate `rustc_middle`
 --> src/main.rs:7:5
  |
7 |     rustc_public::run!(&rustc_args, todo!());
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0432]: unresolved import `rustc_driver`
 --> src/main.rs:7:5
  |
7 |     rustc_public::run!(&rustc_args, todo!());
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0432]: unresolved import `rustc_interface`
 --> src/main.rs:7:5
  |
7 |     rustc_public::run!(&rustc_args, todo!());
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

  </div>

</template>

<template #right>

  <div class="flex h-full items-center justify-center">
    <div class="text-2xl border border-green-800 px-2 py-1">
按需引入编译器模块
    </div>
  </div>

</template>

</TwoColumns>

---

```rust
#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run!(&rustc_args, || ControlFlow::<(), ()>::Break(()));
}
```

<div class="CodeblockTitle">
最简入口代码
</div>

<v-click>

```bash
$ cargo run -- src/main.rs # 编译检查工具，并分析 main.rs
     Running `target/debug/redpen src/main.rs`
```

</v-click>

---
hideInToc: true
---

### `run!` 与回调函数

```rust
pub struct AnalysisFn<B, C, F> { ... }

impl<B, C, F> rustc_driver::Callbacks for AnalysisFn<B, C, F>
where
    B: Send, C: Send,
    F: FnOnce($($crate::optional!($with_tcx TyCtxt))?) -> ControlFlow<B, C> + Send,
{
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        // Run the analysis function `F` and returns
        // Compilation::Continue or Compilation::Stop.
    }
}
```

<div class="CodeblockTitle">

`run!` 和 `run_with_tcx!` 宏展开

</div>

---
hideInToc: true
---

### `run!` 与回调函数

```rust {1,17|4|2|7-11}
pub fn run(&mut self, args: &[String]) -> Result<C, CompilerError<B>> {
    let compiler_result = rustc_driver::catch_fatal_errors(
        || -> interface::Result::<()> {
          run_compiler(&args, self);
          Ok(())
    });
    match (compiler_result, self.result.take()) {
        (Ok(Ok(())), Some(ControlFlow::Continue(value))) => Ok(value),
        (Ok(Ok(())), Some(ControlFlow::Break(value))) => {
            Err(CompilerError::Interrupted(value))
        }
        (Ok(Ok(_)), None) => Err(CompilerError::Skipped),
        (Ok(Err(_)), _) | (Err(_), _) => Err(CompilerError::Failed),
    }
}

AnalysisFn::new($callback).run($args)
```

<div class="CodeblockTitle" style="padding: 0rem">

`run!` 和 `run_with_tcx!` 宏展开

</div>

---
hideInToc: true
---

### `run!` / `run_with_tcx!` 与返回值类型


```rust
run!(&rustc_args, || -> ControlFlow<B, C> { ... })
run_with_tcx!(&rustc_args, |tcx| -> ControlFlow<B, C> { ... })
```

<Rect :x="230" :y="-60" :w="180" :h="25" v-click="[1, 2]" />
<Rect :x="345" :y="-35" :w="180" :h="25" v-click="[1, 2]" />

<v-click at="1">

闭包返回 `enum ControlFlow::<B, C> { Break(B), Continue(C) }`：

* `Continue(C)`：继续编译（分析之后生成二进制文件）
* `Break(B)`：终止编译

</v-click>

```rust
let res: Result<C, CompilerError<B>> = run!(...);
let res: Result<C, CompilerError<B>> = run_with_tcx!(...);
```

<Rect :x="88" :y="-61" :w="270" :h="50" v-click="[2, 3]" />

<v-click at="2">

宏返回 `Result`：

* `Ok(C)`：编译**正常**完成，得到闭包的结果
* `Err(CompilerError::Interrupted(B))`：编译**正常**终止，得到闭包的结果
* `Err(CompilerError::Failed)`：编译期间出现错误（ICE）
* `Err(CompilerError::Skipped)`：没有真正编译代码，比如调用 -v 获取信息

</v-click>

--- 

### 程序分析函数

<CodeblockSmallSized>

```rust {*}{lines: true}
_ = run!(&rustc_args, analysis);
```

```rust {*|4|6|7,14,15}{lines:true}
use rustc_public::{CrateDef, CrateItem, mir::mono::Instance, ty::FnDef};

fn analysis() -> ControlFlow<(), ()> {
    let local_crate = rustc_public::local_crate();
    let crate_name = &*local_crate.name;
    for f in local_crate.fn_defs() {
        let inst_name = name_from_instance(&f);
        println!("[{crate_name}] {:33} - (instance) {inst_name}", f.name());
    }
    
    ControlFlow::Break(())
}

fn name_from_instance(f: &FnDef) -> String {
    Instance::try_from(CrateItem(f.def_id()))
        .map(|inst| inst.name())
        .unwrap_or_else(|err| err.to_string())
}
```

</CodeblockSmallSized>

<TwoColumns>

<template #left>

* [`rustc_public::local_crate`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/fn.local_crate.html)
* [`fn fn_defs(&self) -> Vec<FnDef>`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/struct.Crate.html#method.fn_defs)

</template>

<template #right>

* [`rustc_public::mir::mono::Instance`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/mir/mono/struct.Instance.html)
* [`rustc_public::DefId`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/crate_def/struct.DefId.html)

</template>

</TwoColumns>

<!-- <Line x1="220" y1="192" x2="425" v-click="[1,2]" /> -->
<!-- <Line x1="158" y1="226" x2="312" v-click="[2,3]" /> -->
<!-- <Line x1="92"  y1="390" x2="390" v-click="[3,4]" /> -->

---

<CodeblockSmallSized>

```bash
$ cargo run --example print-fn-names -- src/main.rs
     Running `target/debug/examples/print-fn-names src/main.rs`
[main] main                              - (instance) main
[main] main::RustcPublic::<B, C, F>::new - (instance) Item requires monomorphization
[main] main::RustcPublic::<B, C, F>::run - (instance) Item requires monomorphization
[main] <main::RustcPublic<B, C, F> as rustc_driver::Callbacks>::after_analysis - (instance) Item requires monomorphization
```

<v-click>

<Info>
🤔 Item requires <strong>monomorphization （单态化）</strong>?
</Info>

</v-click>
</CodeblockSmallSized>

---

## Monomorphization

<div class="h-4" />

> 编译器会为所需的每个具体类型标记出泛型函数代码的不同副本。
>
> 例如，如果代码中使用了 `Vec<u64>` 和 `Vec<String>`，则生成的二进制文件将生成两个 Vec 副本。
>
> 最终生成高效的程序，但具有一些代码：
> * 编译时间（创建所有这些副本可能需要一段时间）
> * 二进制大小（所有这些副本可能占用大量空间）

<CodeblockSmallSized>
<TwoColumns>

<template #right>

```rust
pub enum RigidTy { // 类似 TyKind 的结构
    Adt(AdtDef, GenericArgs),
    FnDef(FnDef, GenericArgs),
    Closure(ClosureDef, GenericArgs),
    ...
}
```

</template>

<template #left>

rustc_public 的单态化表示：

```rust
// Instance = FnDef + 填充具体类型的参数
fn resolve(def: FnDef, args: &GenericArgs) -> Result<Instance, Error> { }
```

</template>

</TwoColumns>
</CodeblockSmallSized>

编译器中单态化的细节：

* [rustc-dev-guide#monomorph](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html)
* [rustc_monomorphize::collector](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_monomorphize/collector/index.html)
* [rustc_middle::ty::InstanceKind](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.InstanceKind.html)
