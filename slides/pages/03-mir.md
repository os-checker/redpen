
## MIR

<TwoColumns>

<template #left>

<div class="bg-[#f5f5f5] flex items-center justify-center flex-col">

![](https://blog.rust-lang.org/2016/04/19/MIR/flow.svg)

</div>

<div class="text-center">

《[Introducing MIR](https://blog.rust-lang.org/2016/04/19/MIR/)》(Rust Blog, 2016)

</div>

</template>

<template #right>

* 完整 MIR Body：[`rustc_middle::mir::Body`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Body.html)
* 精简 MIR Body：[`rustc_public::mir::Body`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/mir/struct.Body.html)

<div class="text-base">

MIR = Middle-Level Intermediate Representation
([Ref](https://rustc-dev-guide.rust-lang.org/mir/index.html))

</div>

```rust
pub struct Body {
    blocks: Vec<BasicBlock>,
    locals: Vec<LocalDecl>,
    arg_count: usize,
    var_debug_info: Vec<VarDebugInfo>,
    spread_arg: Option<Local>,
    span: Span,
}
```

<div class="text-center">

[`rustc_public::mir::Body`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/mir/struct.Body.html)

</div>

</template>

</TwoColumns>

---

### MIR Body (精简版)

<div class="h-2" />

<CodeblockSmallSized>

<TwoColumns>

<template #left>

<div class="border border-orange-200 px-1">

```rust
pub struct LocalDecl {
    ty: Ty,
    span: Span,
    mutability: Mutability,
}
```

</div>
<div class="text-sm">

`Body.locals: Vec<LocalDecl>` 类似函数的栈，存放不同的值

* 第 0 个位置为返回值
* 第 `1..=arg_count` 位置为函数参数
* 剩余位置存放使用者定义的变量和临时变量

</div>

</template>

<template #right>

<div class="border border-green-200 px-1">

```rust
pub struct VarDebugInfo {
    name: Symbol,
    source_info: SourceInfo,
    composite: Option<VarDebugInfoFragment>,
    value: VarDebugInfoContents,
    argument_index: Option<u16>,
}
```

</div>

<p class="text-sm">
局部变量的名称、源码位置、类型、值、可能的函数参数位置。
</p>

</template>

</TwoColumns>

<v-click>

<div class="flex items-center justify-between gap-4 py-4">

<div>

[Demo](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=abd4c3f618b81d9f241db11efee09b80)

</div>

<div class="flex-1">

```rust
let mut vec = Vec::new();
vec.push(1);
```

</div>

</div>

<TwoColumns>

<template #left>

<div class="border border-orange-200 px-1">

```rust
let mut _0: ();
let mut _1: std::vec::Vec<i32>;
let _2: ();
let mut _3: &mut std::vec::Vec<i32>;
```

</div>
</template>

<template #right>

<div class="border border-green-200 px-1">

```rust
scope 1 {
    debug vec => _1;
}
```

</div>
</template>

</TwoColumns>

<div class="text-center">Dump MIR</div>

</v-click>
</CodeblockSmallSized>

---

<CodeblockSmallSized>

<TwoColumns>

<template #left>

```rust
pub struct BasicBlock {
    statements: Vec<Statement>,
    terminator: Terminator,
}
```

```rust
pub enum StatementKind {
    Assign(Place, Rvalue),
    FakeRead(FakeReadCause, Place),
    SetDiscriminant {
        place: Place,
        variant_index: VariantIdx,
    },
    StorageLive(Local),
    StorageDead(Local),
    Retag(RetagKind, Place),
    PlaceMention(Place),
    AscribeUserType {
        place: Place,
        projections: UserTypeProjection,
        variance: Variance,
    },
    Coverage(Opaque),
    Intrinsic(NonDivergingIntrinsic),
    ConstEvalCounter,
    Nop,
}
```

</template>

<template #right>

```rust
pub enum TerminatorKind {
    Goto { target: BasicBlockIdx, },
    SwitchInt { discr: Operand, targets: SwitchTargets },
    Resume, Abort, Return, Unreachable,
    Drop { place: Place, target: BasicBlockIdx, unwind: UnwindAction },
    Call {
        func: Operand, args: Vec<Operand>, destination: Place,
        target: Option<BasicBlockIdx>, unwind: UnwindAction,
    },
    Assert {
        cond: Operand, expected: bool, msg: AssertMessage,
        target: BasicBlockIdx, unwind: UnwindAction,
    },
    InlineAsm { ... },
}
```

</template>

</TwoColumns>

</CodeblockSmallSized>

---

```rust
bb0: {
    _1 = Vec::<i32>::new() -> [return: bb1, unwind continue];
}

bb1: {
    _3 = &mut _1;
    _2 = Vec::<i32>::push(move _3, const 1_i32) -> [return: bb2, unwind: bb4];
}

bb2: {
    drop(_1) -> [return: bb3, unwind continue];
}

bb3: {
    return;
}

bb4 (cleanup): {
    drop(_1) -> [return: bb5, unwind terminate(cleanup)];
}

bb5 (cleanup): {
    resume;
}
```
