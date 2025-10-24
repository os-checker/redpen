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
