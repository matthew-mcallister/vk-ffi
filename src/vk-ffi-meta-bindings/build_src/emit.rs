use proc_macro2::TokenStream;

use super::Defs;

#[derive(Debug)]
crate struct EmittedFile {
    crate file_name: &'static str,
    crate tokens: TokenStream,
}

macro_rules! impl_emit {
    ($defs:ident; $($member:ident: $type:ty,)*) => {
        let bang: Token![!] = Default::default();
        $(
            let $member = $defs.$member;
            let file_name = concat!(stringify!($member), ".rs");
            let tokens = quote! {
                pub fn $member() -> impl Iterator<Item = $type> {
                    use ::syn::punctuated::Punctuated;
                    let punct: Punctuated<$type, Token #bang [;]> =
                        parse_quote #bang { #(#$member;)* };
                    punct.into_iter()
                }
            };
            let $member = EmittedFile { file_name, tokens };
        )*
        vec![$($member,)*]
    }
}

crate fn emit_defs(defs: Defs) -> Vec<EmittedFile> {
    impl_emit! {
        defs;
        enums: Enum,
        consts: Const,
        structs: Struct,
        unions: Union,
        fn_pointers: FnPointer,
        type_aliases: TypeAlias,
        handles: Handle,
    }
}
