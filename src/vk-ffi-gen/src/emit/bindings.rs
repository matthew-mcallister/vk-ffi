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
        quote!(pub #name: #ty,)
    });
    quote! {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct #struct_name { #(#members)* }
    }
}

fn emit_union(def: &Union) -> TokenStream {
    let union_name = &def.0.name;
    let members = def.0.members.iter().map(|member| {
        let name = &member.name;
        let ty = &member.ty;
        quote!(pub #name: #ty,)
    });
    quote! {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub union #union_name { #(#members)* }
    }
}

fn emit_fn_pointer(def: &FnPointer) -> TokenStream {
    let base_name = &def.base_name;
    let fn_name = map_ident(base_name, |s| format!("Fn{}", s));
    let pfn_name = map_ident(base_name, |s| format!("Pfn{}", s));
    let signature = &def.signature;
    quote! {
        pub type #fn_name = #signature;
        pub type #pfn_name = ::std::option::Option<#signature>;
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
            impl #name {
                pub fn null() -> Self { #name(0 as *const _) }
                pub fn is_null(self) -> bool { self.0 as usize == 0 }
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
            impl #name {
                pub fn null() -> Self { #name(0) }
                pub fn is_null(self) -> bool { self.0 == 0 }
            }
        }
    }
}

crate fn emit(defs: &Defs) -> TokenStream {
    let mut tokens = TokenStream::new();
    for def in defs.enums.iter() { tokens.extend(emit_enum(&def)); }
    for def in defs.consts.iter() { tokens.extend(emit_const(&def)); }
    for def in defs.structs.iter() { tokens.extend(emit_struct(&def)); }
    for def in defs.unions.iter() { tokens.extend(emit_union(&def)); }
    for def in defs.fn_pointers.iter()
        { tokens.extend(emit_fn_pointer(&def)); }
    for def in defs.type_aliases.iter()
        { tokens.extend(emit_type_alias(&def)); }
    for def in defs.handles.iter() { tokens.extend(emit_handle(&def)); }
    tokens
}
