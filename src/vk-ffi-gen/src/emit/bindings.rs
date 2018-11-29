use proc_macro2::TokenStream;

use crate::map_ident;
use crate::defs::*;

fn emit_enum(def: &Enum) -> TokenStream {
    let enum_name = &def.name;
    let ty = &def.ty;
    let members = def.members.iter().map(|member| {
        let name = &member.name;
        let value = &member.value;
        quote!(pub const #name: #enum_name = #enum_name(#value);)
    });
    let mut tokens = quote! {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct #enum_name(pub #ty);
        impl #enum_name { #(#members)* }
    };
    if def.is_bitmask() {
        let bang: Token![!] = Default::default();
        tokens.extend(quote!(bitmask_impls #bang (#enum_name);));
    }
    tokens
}

fn emit_const(def: &Const) -> TokenStream {
    let name = &def.name;
    let ty = &def.ty;
    let value = &def.value;
    quote!(pub const #name: #ty = #value;)
}

fn emit_struct(def: &Struct) -> TokenStream {
    let struct_name = &def.name;
    let members = def.members.iter().map(|member| {
        let name = &member.name;
        let ty = &member.ty;
        quote!(pub #name: #ty)
    });
    quote! {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct #struct_name { #(#members,)* }
    }
}

fn emit_struct_traits(defs: &Defs, def: &Struct) -> TokenStream {
    let mut tokens = TokenStream::new();
    let struct_name = &def.name;

    let members = def.members.iter().map(|member| {
        let name = &member.name;
        let value: Option<_> = try {
            // If the field name is `s_type`, attempt to look up the
            // corresponding `VkStructureType` member.
            if name.to_string() != "s_type" { None? }
            let struct_type = defs.get_stype(struct_name.to_string())?;
            quote!(StructureType::#struct_type)
        };
        let value = value.unwrap_or_else(|| {
            // For other fields, use the default you'd expect from C.
            match &member.ty {
                syn::Type::Ptr(syn::TypePtr { mutability: Some(_), .. }) =>
                    quote!(::std::ptr::null_mut()),
                syn::Type::Ptr(syn::TypePtr { mutability: None, .. }) =>
                    quote!(::std::ptr::null()),
                syn::Type::Array(syn::TypeArray { ref len, .. }) =>
                    quote!([::std::default::Default::default(); #len]),
                _ => quote!(::std::default::Default::default()),
            }
        });
        quote!(#name: #value)
    });
    tokens.extend(quote! {
        impl ::std::default::Default for #struct_name {
            fn default() -> Self {
                #struct_name { #(#members,)* }
            }
        }
    });

    tokens
}

fn emit_union(def: &Union) -> TokenStream {
    let union_name = &def.0.name;
    let members = def.0.members.iter().map(|member| {
        let name = &member.name;
        let ty = &member.ty;
        quote!(pub #name: #ty)
    });
    quote! {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub union #union_name { #(#members,)* }
    }
}

/// Implements `Debug` and `Default` for unions. `Debug` is given a
/// trivial implementation. `Default` is implemented similarly to
/// default union initialization in C: the first member is
/// default-initialized.
fn emit_union_traits(def: &Union) -> TokenStream {
    let mut tokens = TokenStream::new();
    let union_name = &def.0.name;

    tokens.extend(quote! {
        impl ::std::fmt::Debug for #union_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
            {
                write!(f, concat!(stringify!(#union_name), " {{ (union) }}"))
            }
        }
    });

    let first = &def.0.members[0].name;
    tokens.extend(quote! {
        impl ::std::default::Default for #union_name {
            fn default() -> Self {
                #union_name { #first: ::std::default::Default::default() }
            }
        }
    });

    tokens
}

fn emit_fn_pointer(def: &FnPointer) -> TokenStream {
    let fn_name = def.fn_name();
    let pfn_name = def.pfn_name();
    let signature = &def.signature;
    quote! {
        pub type #fn_name = #signature;
        pub type #pfn_name = ::std::option::Option<#fn_name>;
    }
}

fn emit_type_alias(def: &TypeAlias) -> TokenStream {
    let name = &def.name;
    let target = &def.target;
    quote!(pub type #name = #target;)
}

fn emit_handle(def: &Handle) -> TokenStream {
    let name = &def.name;
    let name_t = map_ident(name, |s| format!("{}T", s));
    if def.dispatchable {
        quote! {
            extern { pub type #name_t; }
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
            pub struct #name(pub *const #name_t);
            impl VkHandle for #name {
                fn null() -> Self { #name(0 as *const _) }
                fn is_null(self) -> bool { self.0 as usize == 0 }
            }
            impl ::std::default::Default for #name {
                fn default() -> Self { #name::null() }
            }
        }
    } else {
        quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
            pub struct #name(pub u64);
            impl VkHandle for #name {
                fn null() -> Self { #name(0) }
                fn is_null(self) -> bool { self.0 == 0 }
            }
        }
    }
}

crate fn emit(defs: &Defs) -> TokenStream {
    let mut tokens = TokenStream::new();
    for def in defs.enums.iter() { tokens.extend(emit_enum(def)); }
    for def in defs.consts.iter() { tokens.extend(emit_const(def)); }
    for def in defs.structs.iter() {
        tokens.extend(emit_struct(def));
        tokens.extend(emit_struct_traits(defs, def));
    }
    for def in defs.unions.iter() {
        tokens.extend(emit_union(def));
        tokens.extend(emit_union_traits(def));
    }
    for def in defs.fn_pointers.iter()
        { tokens.extend(emit_fn_pointer(def)); }
    for def in defs.type_aliases.iter()
        { tokens.extend(emit_type_alias(def)); }
    for def in defs.handles.iter() { tokens.extend(emit_handle(def)); }
    tokens
}
