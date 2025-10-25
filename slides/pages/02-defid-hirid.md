# Rust 编译器中间表示

<div class="columns-2">
<Toc mode="onlyCurrentTree" />
</div>

<BackToTOC />

---

## `DefId` 和 `HirId`

<div class="h-4" />

<TwoColumns gap="2rem">

<template #left>

<div class="flex flex-col items-center justify-center">

<div>ID for a Crate Item Definition</div>

```rust
// This ID can represent a local or external item.
pub struct DefId {
    pub index: DefIndex,           // (*)
    pub krate: CrateNum,
}

// DefId.krate (CrateNum) == LOCAL_CRATE == 0
pub struct LocalDefId {
    pub local_def_index: DefIndex, // (*)
}
struct DefIndex(u32);              // (*)
```

</div>

</template>

<template #right>

<div class="flex flex-col items-center justify-center">

<div>ID for a Local Crate Item and Expression</div>

```rust
// This represents a local node.
pub struct HirId {
    pub owner: OwnerId, // (1)
    pub local_id: ItemLocalId, // (2)
}

pub struct OwnerId {    // (1)
    pub def_id: LocalDefId,
}

pub struct ItemLocalId(u32);   // (2)
```

</div>

</template>

</TwoColumns>

<div class="h-4" />

Ref：<https://rustc-dev-guide.rust-lang.org/hir.html#identifiers-in-the-hir>

(注 `rustc_public::DefId` 使用 usize 索引，并与 `DefId` 相互转化。)

---

### `AdtDef` 与 `DefId`

<CodeblockSmallSized>

```rust
struct MyStruct<T> { x: u8, y: T }

// The type MyStruct<u32> would be an instance of TyKind::Adt:

Adt(&'tcx AdtDef, GenericArgs<'tcx>)
//  ------------  ---------------
//  (1)            (2)

// (1) represents the `MyStruct` part
// (2) represents the `<u32>`, or "substitutions" / generic arguments
```

```rust
pub struct AdtDef<'tcx>(&'tcx AdtDefData);
pub struct AdtDefData {
    // The DefId of the struct, enum or union item.
    pub did: DefId, // 👀
    // Variants of the ADT. If this is a struct or union, then there will be a single variant.
    variants: IndexVec<VariantIdx, VariantDef>, // 👀 VariantDef 也含有 DefId
    // Flags of the ADT (e.g., is this a struct, enum or union? is this non-exhaustive?).
    flags: AdtFlags,
    // Repr options provided by the user. (#[repr(...)])
    repr: ReprOptions,
}
```

</CodeblockSmallSized>

[Ref](https://rustc-dev-guide.rust-lang.org/ty_module/generic_arguments.html).
ADT (algebraic data type) = user-defined type, e.g., a struct, enum, or union.


---

### HIR `Node`

<ol class="columns-4">

<li>Param</li>
<li>Item</li>
<li>ForeignItem</li>
<li>TraitItem</li>
<li>ImplItem</li>
<li>Variant</li>
<li>Field</li>
<li>AnonConst</li>
<li>ConstBlock</li>
<li>ConstArg</li>
<li>Expr</li>
<li>ExprField</li>
<li>Stmt</li>
<li>PathSegment</li>
<li>Ty</li>
<li>AssocItemConstraint</li>
<li>TraitRef</li>
<li>OpaqueTy</li>
<li>TyPat</li>
<li>Pat</li>
<li>PatField</li>
<li>PatExpr</li>
<li>Arm</li>
<li>Block</li>
<li>LetStmt</li>
<li>Ctor</li>
<li>Lifetime</li>
<li>GenericParam</li>
<li>Crate</li>
<li>Infer</li>
<li>WherePredicate</li>
<li>PreciseCapturing...</li>
<li>Synthetic</li>
<li>Err</li>

</ol>

<CodeblockSmallSized>

<div class="h-2" />

<TwoColumns>

<template #left>

```rust
pub struct Stmt<'hir> {
    pub hir_id: HirId,
    pub kind: StmtKind<'hir>,
    pub span: Span,
}
```

</template>

<template #right>

```rust
pub enum StmtKind<'hir> {
    Let(&'hir LetStmt<'hir>),
    Item(ItemId),
    Expr(&'hir Expr<'hir>),
    Semi(&'hir Expr<'hir>),
}
```

</template>

</TwoColumns>

</CodeblockSmallSized>

---

### HIR `Expr`


```rust
pub struct Expr<'hir> {
    pub hir_id: HirId,
    pub kind: ExprKind<'hir>,
    pub span: Span,
}
```

[Ref](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Expr.html). 其中 enum `ExprKind`：

<ol class="columns-5">
<li>ConstBlock</li>
<li>Array</li>
<li>Call</li>
<li>MethodCall</li>
<li>Use</li>
<li>Tup</li>
<li>Binary</li>
<li>Unary</li>
<li>Lit</li>
<li>Cast</li>
<li>Type</li>
<li>DropTemps</li>
<li>Let</li>
<li>If</li>
<li>Loop</li>
<li>Match</li>
<li>Closure</li>
<li>Block</li>
<li>Assign</li>
<li>AssignOp</li>
<li>Field</li>
<li>Index</li>
<li>Path</li>
<li>AddrOf</li>
<li>Break</li>
<li>Continue</li>
<li>Ret</li>
<li>Become</li>
<li>InlineAsm</li>
<li>OffsetOf</li>
<li>Struct</li>
<li>Repeat</li>
<li>Yield</li>
<li>UnsafeBinderCast</li>
<li>Err</li>
</ol>


---

### AST -> HIR

HIR 对 AST 的结构进行了清理，删除了表层语法等无关类型分析的结构。
([Ref](https://rustc-dev-guide.rust-lang.org/hir/lowering.html))

* 各种括号
* for 循环：转化为 `match + loop + match` 结构
* Universal `impl Trait` （函数入参位置）：转化为泛型参数
* Existential `impl Trait` （函数返回值位置）：转化为虚拟的 `existential type` 声明

<div class="h-1" />

### HIR -> THIR

THIR = Typed High-Level Intermediate Representation ([Ref](https://rustc-dev-guide.rust-lang.org/thir.html))

* HIR 节点中所有类型填充完毕（通过了类型检查）
* 以 body 方式呈现（比如函数体、 const initializer）
* 只有可执行代码，不再有 struct、trait 内定义的项
* 每个 THIR body 只会被临时存储，随 HIR 只存在当前的被编译 crate 中
* 额外的脱糖：显式自动引用和取消引用、方法调用和重载运算符被转换为普通函数调用、显式的销毁范围
* 语句、表达式和 match arm 单独存储：stmts 数组中，每个表达式通过 ExprId 来索引 exprs 数组的表达式

---

### 比较不同编译时期的 body

<CodeblockSmallSized>

<TwoColumns>

<template #left>

```rust
pub struct Block {
    pub stmts: ThinVec<Stmt>,
    // ast 的 NodeId 是 HirId、DefId 的基础
    pub id: NodeId,
}

pub enum StmtKind { // ast::Stmt.kind
    Let(Box<Local>),
    Item(Box<Item>),
    Expr(Box<Expr>),
    Semi(Box<Expr>),
    Empty,
    MacCall(Box<MacCallStmt>),
}
```

<div class="text-center">AST Body</div>

<div class="h-4" />

```rust
pub struct Body<'hir> {
    pub params: &'hir [Param<'hir>],
    pub value: &'hir Expr<'hir>,
}
```

<div class="text-center">HIR Body</div>

</template>

<template #right>

```rust
pub struct Thir<'tcx> {
    pub body_type: BodyTy<'tcx>,
    pub arms: IndexVec<ArmId, Arm<'tcx>>,
    pub blocks: IndexVec<BlockId, Block>,
    pub exprs: IndexVec<ExprId, Expr<'tcx>>,
    pub stmts: IndexVec<StmtId, Stmt<'tcx>>,
    pub params: IndexVec<ParamId, Param<'tcx>>,
}

pub enum BodyTy<'tcx> {
    Const(Ty<'tcx>),
    Fn(FnSig<'tcx>),
    GlobalAsm(Ty<'tcx>),
}

pub enum StmtKind<'tcx> {
    Expr { scope: Scope, expr: ExprId, },
    Let  {
        init_scope: Scope,
        initializer: Option<ExprId>,
        ...
    },
}
```

<div class="text-center">THIR Body</div>

</template>

</TwoColumns>

</CodeblockSmallSized>
