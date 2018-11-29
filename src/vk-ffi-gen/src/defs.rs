use std::collections::HashMap;

use heck::*;

use crate::{map_ident, to_slug};

#[derive(Clone, Debug)]
crate struct Enum {
    crate name: syn::Ident,
    crate ty: Box<syn::Type>,
    crate members: Vec<EnumMember>,
}

impl Enum {
    crate fn is_bitmask(&self) -> bool {
        self.name.to_string().contains("FlagBits")
    }
}

#[derive(Clone, Debug)]
crate struct EnumMember {
    crate name: syn::Ident,
    crate value: Box<syn::Expr>,
}

#[derive(Clone, Debug)]
crate struct Const {
    crate name: syn::Ident,
    crate ty: Box<syn::Type>,
    crate value: Box<syn::Expr>,
}

#[derive(Clone, Debug)]
crate struct Struct {
    crate name: syn::Ident,
    crate members: Vec<StructMember>
}

#[derive(Clone, Debug)]
crate struct StructMember {
    crate name: syn::Ident,
    crate ty: syn::Type,
}

#[derive(Clone, Debug)]
crate struct Union(crate Struct);

#[derive(Clone, Debug)]
crate struct FnPointer {
    /// The "base" name of the function, with the "PFN_vk" prefix
    /// removed, but with the case unconverted.
    crate base_name: syn::Ident,
    crate signature: syn::TypeBareFn,
}

impl FnPointer {
    crate fn pfn_name(&self) -> syn::Ident {
        map_ident(&self.base_name, |s| format!("Pfn{}", s.to_camel_case()))
    }

    crate fn fn_name(&self) -> syn::Ident {
        map_ident(&self.base_name, |s| format!("Fn{}", s.to_camel_case()))
    }

    /// The name of the symbol corresponding to a command.
    crate fn symbol_name(&self) -> String {
        format!("vk{}", self.base_name.to_string())
    }

    /// For good measure, the base name converted to snake case.
    crate fn snake_name(&self) -> syn::Ident {
        map_ident(&self.base_name, |s| s.to_snake_case())
    }
}

#[derive(Clone, Debug)]
crate struct TypeAlias {
    crate name: syn::Ident,
    crate target: syn::Path,
}

#[derive(Clone, Debug)]
crate struct Handle {
    crate name: syn::Ident,
    crate dispatchable: bool,
}

#[derive(Debug, Default)]
crate struct Defs {
    crate enums: Vec<Enum>,
    crate consts: Vec<Const>,
    crate structs: Vec<Struct>,
    crate unions: Vec<Union>,
    crate fn_pointers: Vec<FnPointer>,
    crate type_aliases: Vec<TypeAlias>,
    crate handles: Vec<Handle>,
    crate stype_map: HashMap<String, syn::Ident>,
}

impl Defs {
    crate fn get_stype<S: Into<String>>(&self, name: S) ->
        Option<&syn::Ident>
    {
        self.stype_map.get(&to_slug(&name.into()))
    }
}

#[derive(Clone, Debug, From)]
crate enum Def {
    Enum(Enum),
    Const(Const),
    Struct(Struct),
    Union(Union),
    FnPointer(FnPointer),
    TypeAlias(TypeAlias),
    Handle(Handle),
}
