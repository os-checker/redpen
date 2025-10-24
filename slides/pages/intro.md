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

### `run!` 与回调函数


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

<!-- <div v-click="[2, 3]" v-motion -->
<!--   :initial="{ x: -80, opacity: 0 }" -->
<!--   :enter="{ x: 88, y: -61, opacity: 1 }" -->
<!--   :leave="{ x: 100, opacity: 0 }" -->
<!--   class="absolute w-[268px] h-[50px] -->
<!--          pointer-events-none -->
<!--          border-2 border-amber-500 -->
<!--          bg-amber-500/5 rounded-md"/> -->

<v-click at="2">

宏返回 `Result`：

* `Ok(C)`：编译正常完成，得到闭包的结果
* `Err(CompilerError::Interrupted(B))`：编译正常终止，得到闭包的结果
* `Err(CompilerError::Failed)`：编译期间出现错误（ICE）
* `Err(CompilerError::Skipped)`：没有真正编译代码，比如调用 -v 获取信息

</v-click>

