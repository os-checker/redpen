# Redpen Checker

<!-- <div class="columns-2"> -->
<Toc mode="onlyCurrentTree" />
<!-- </div> -->

<BackToTOC />

---

## Check Panics

目标 1：找出所有到达的 `panic` 调用。

* 可达性分析（基于 MirVisitor 示例）
* 打印调用发生的位置
* 打印调用发生的路径


<!-- [MirVisitor::super_body](https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_public/mir/visit.rs.html#387) 源码 -->



---

References:

* [Rust Dev Guide: Diagnostics](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
* [Clippy Development](https://doc.rust-lang.org/clippy/development/index.html)
* [`rustc_middle::lint::lint_level`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/lint/fn.lint_level.html) to emit lints

