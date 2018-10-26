use super::map_ident;

crate fn do_rewrite(bindings: &mut syn::File) {
    for idx in 0..bindings.items.len() {
        let ty_def = match bindings.items[idx] {
            syn::Item::Type(ref ty_def) => ty_def,
            _ => continue,
        };

        let dispatchable = match ty_def.ty {
            box syn::Type::Ptr(_) => true,
            box syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let target_name = path.segments.last().unwrap().into_value()
                    .ident.to_string();
                if target_name == "NonDispatchableHandleVkFfi" { false }
                else { continue }
            }
            _ => panic!("unexpected typedef"),
        };

        let name = ty_def.ident.clone();
        let ty = if dispatchable {
            let target = map_ident(|mut s| { s.push('T'); s }, name.clone());
            quote!(*mut #target)
        } else { quote!(u64) };
        let def: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
            pub struct #name(pub #ty);
        };
        bindings.items[idx] = def.into();
    }
}
