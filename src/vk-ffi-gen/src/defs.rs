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
    /// The "base" name of the function, i.e. without any prefix such as
    /// `Pfn`.
    crate base_name: syn::Ident,
    crate signature: syn::TypeBareFn,
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
