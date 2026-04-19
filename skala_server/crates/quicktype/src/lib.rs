use std::fmt::Display;

pub use linkme;
pub use quicktype_macros::Quicktype;

use linkme::distributed_slice;

pub trait Quicktype {
    fn type_name() -> TypeName;
    fn type_spec() -> TypeSpec;
}

pub fn derived_type() -> impl Iterator<Item = &'static QuicktypeDerivedType> {
    DEFINITIONS.into_iter()
}

#[doc(hidden)]
#[distributed_slice]
pub static DEFINITIONS: [QuicktypeDerivedType];

#[derive(Clone, Debug)]
pub struct QuicktypeDerivedType {
    pub name: TypeName,
    pub spec: TypeSpec,
}

#[derive(Clone, Debug)]
pub struct TypeName {
    pub namespace: Option<Namespace>,
    pub unqualified_name: UnqualifiedTypeName,
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            namespace,
            unqualified_name,
        } = self;
        if let Some(namespace) = namespace {
            write!(f, "{namespace}.{unqualified_name}")
        } else {
            write!(f, "{unqualified_name}")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Namespace(&'static str);

impl Namespace {
    pub const fn new(inner: &'static str) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> &'static str {
        let Self(inner) = self;
        inner
    }
}

impl From<&'static str> for Namespace {
    fn from(inner: &'static str) -> Self {
        Self::new(inner)
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        write!(f, "{inner}")
    }
}

#[derive(Clone, Debug)]
pub struct UnqualifiedTypeName(&'static str);

impl UnqualifiedTypeName {
    pub const fn new(inner: &'static str) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> &'static str {
        let Self(inner) = self;
        inner
    }
}

impl From<&'static str> for UnqualifiedTypeName {
    fn from(inner: &'static str) -> Self {
        Self::new(inner)
    }
}

impl Display for UnqualifiedTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        write!(f, "{inner}")
    }
}

#[derive(Clone, Debug)]
pub struct TypeSpec(&'static str);

impl TypeSpec {
    pub const fn new(spec: &'static str) -> Self {
        Self(spec)
    }

    pub fn into_inner(self) -> &'static str {
        let Self(inner) = self;
        inner
    }
}

impl From<&'static str> for TypeSpec {
    fn from(inner: &'static str) -> Self {
        Self::new(inner)
    }
}

impl Display for TypeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        write!(f, "{inner}")
    }
}
