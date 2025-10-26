use rustc_public::{CrateDef, ty::FnDef};
use std::{fmt, rc::Rc, sync::LazyLock};

/// A FnDef simplified on Debug trait and `{:?}` printing.
#[derive(Clone)]
pub struct FnItem {
    pub def: FnDef,
    // This field is for debug purpose.
    // FnDef is enough for comparing and hashing.
    pub name: Rc<str>,
}
impl PartialEq for FnItem {
    fn eq(&self, other: &Self) -> bool {
        self.def == other.def
    }
}
impl Eq for FnItem {}
impl std::hash::Hash for FnItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.def.hash(state);
    }
}
impl fmt::Debug for FnItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}
impl PartialOrd for FnItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FnItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
impl From<FnDef> for FnItem {
    fn from(fn_def: FnDef) -> Self {
        Self::new(fn_def)
    }
}
impl FnItem {
    pub fn new(fn_def: FnDef) -> Self {
        FnItem {
            def: fn_def,
            name: fn_def.name().into(),
        }
    }

    pub fn is(&self, name: &str) -> bool {
        *self.name == *name
    }

    pub fn print(&self) -> String {
        // .diagnostic() contain absolute sysroot path, thus strip to shorten it
        static PREFIX: LazyLock<Box<str>> = LazyLock::new(|| {
            let output = std::process::Command::new("rustc")
                .arg("--print=sysroot")
                .output()
                .unwrap();
            assert!(output.status.success());
            let sysroot = std::str::from_utf8(&output.stdout).unwrap().trim();
            format!("{sysroot}/lib/rustlib/src/rust/library/").into()
        });

        let span = self.def.span().diagnostic();
        format!("{} ({})", self.name, span.trim_start_matches(&**PREFIX))
    }
}
