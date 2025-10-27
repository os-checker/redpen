# 编译器内的 Lints 系统

<Toc mode="onlyCurrentTree" />

<BackToTOC />

---

## rustc_driver

<CodeblockSmallSized>

```rust
pub trait Callbacks {
    // Provided methods
    fn config(&mut self, _config: &mut Config) {  }

    fn after_crate_root_parsing(
        &mut self, _compiler: &Compiler, _krate: &mut Crate
    ) -> Compilation {  }

    fn after_expansion<'tcx>(
        &mut self, _compiler: &Compiler, _tcx: TyCtxt<'tcx>
    ) -> Compilation { }

    fn after_analysis<'tcx>(
        &mut self, _compiler: &Compiler, _tcx: TyCtxt<'tcx>
    ) -> Compilation { }
}
```

```rust
pub fn run_compiler(at_args: &[String], callbacks: &mut (dyn Callbacks + Send))
```

</CodeblockSmallSized>

* [Rust Dev Guide: rustc_driver](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html)
* estebank/redpen [示例](https://github.com/estebank/redpen/blob/320cbfd469e0a646d0ec281807deb6b1bf341821/src/main.rs#L103)

---

## Lints

When do lints run?

* Pre-expansion pass: Works on AST nodes before macro expansion. 
* Early lint pass: Works on AST nodes after macro expansion and name resolution, just before AST lowering.
* Late lint pass: Works on HIR nodes, towards the end of analysis (after borrow checking, etc.).
* MIR pass: Works on MIR nodes. 

References:
* [Rust Dev Guide: Diagnostics](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
* [Clippy Development](https://doc.rust-lang.org/clippy/development/index.html)
* [`rustc_middle::lint::lint_level`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/lint/fn.lint_level.html) to emit lints

编译器还有自己的 Diagnostics 系统，和 Lints 紧密集成。

---

## `declare_lint!` & EarlyLintPass

<CodeblockSmallSized>

```rust
// 定义一个 Lint 的名称、级别、描述
declare_lint! { WHILE_TRUE, Warn, "suggest using `loop { }` instead of `while true { }`" }

impl EarlyLintPass for WhileTrue { // 分析代码、触发 Lint
    fn check_expr(&mut self, cx: &EarlyContext<'_>, e: &ast::Expr) {
        if let ast::ExprKind::While(cond, ..) = &e.kind
            && let ast::ExprKind::Lit(ref lit) = pierce_parens(cond).kind
            && let ast::LitKind::Bool(true) = lit.kind
            && !lit.span.from_expansion()

        {
            let condition_span = cx.sess.source_map().guess_head_span(e.span);
            cx.struct_span_lint(WHILE_TRUE, condition_span, |lint| {
                lint.build(fluent::example::use_loop)
                    .span_suggestion_short(
                        condition_span,
                        fluent::example::suggestion,
                        "loop".to_owned(),
                        Applicability::MachineApplicable,
                    )
                    .emit();
            })
        }
    }
}
```

</CodeblockSmallSized>

---

## 注册 Lints

阅读：[rustc-dev-guide#LintStore](https://rustc-dev-guide.rust-lang.org/diagnostics/lintstore.html)

<CodeblockSmallSized>

```rust
rustc_interface::Config {
    register_lints: Option<Box<dyn Fn(&Session, &mut LintStore) + Send + Sync>>
}
```

示例：

```rust
impl Callbacks for MyCallbacks {
    fn config(&mut self, config: &mut Config) {
        config.register_lints = Some(Box::new(move |sess, lint_store| {
            lint_store.register_lints(&[&INCORRECT_ATTRIBUTE, &blocking_async::BLOCKING_ASYNC]);
            lint_store.register_late_pass(|_tcx| Box::new(blocking_async::BlockingAsync));
        }));

    }
}
```

</CodeblockSmallSized>

