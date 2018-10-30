#![feature(uniform_paths)]

extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod imp;

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct Enum {
    pub name: syn::Ident,
    pub ty: Box<syn::Type>,
    pub members: Vec<EnumMember>,
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct EnumMember {
    pub name: syn::Ident,
    pub value: Box<syn::Expr>,
}

impl Enum {
    pub fn is_bitflags(&self) -> bool {
        self.name.to_string().contains("FlagBits")
    }
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct Const {
    pub name: syn::Ident,
    pub ty: Box<syn::Type>,
    pub value: Box<syn::Expr>,
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct Struct {
    pub name: syn::Ident,
    pub members: Vec<StructMember>
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct StructMember {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct Union(pub Struct);

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct FnPointer {
    /// The "base" name of the function, i.e. without any prefix such as
    /// `Pfn`.
    pub base_name: syn::Ident,
    pub signature: syn::TypeBareFn,
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct TypeAlias {
    pub name: syn::Ident,
    pub target: syn::Path,
}

#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
pub struct Handle {
    pub name: syn::Ident,
    pub dispatchable: bool,
}
