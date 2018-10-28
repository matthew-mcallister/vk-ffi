use heck::*;

use super::{strip_prefix, strip_suffix};

// TODO: This needs to be pulled from the registry
const VENDOR_TAGS: &[&'static str] = &[
    "Img", "Amd", "Amdx", "Arm", "Fsl", "Brcm", "Nxp", "Nv", "Nvx", "Viv",
    "Vsi", "Kdab", "Android", "Chromium", "Fuchsia", "Google", "Qcom",
    "Lunarg", "Samsung", "Sec", "Tizen", "Renderdoc", "Nn", "Mvk", "Khr",
    "Khx", "Ext", "Mesa",
];

struct Enum {
    name: syn::Ident,
    ty: syn::Type,
    members: Vec<(syn::Ident, syn::Expr)>,
}

fn strip_vendor_suffix(ident: &str) -> &str {
    VENDOR_TAGS.iter()
        .filter_map(|tag| strip_suffix(ident, tag))
        .next()
        .unwrap_or(ident)
}

fn enum_prefix(ident: &str) -> String {
    let base = strip_suffix(ident, "FlagBits").unwrap_or(ident);
    let mut prefix = base.to_shouty_snake_case();
    prefix.push('_');
    prefix
}

impl Enum {
    fn is_bitmask(&self) -> bool {
        self.name.to_string().contains("FlagBits")
    }

    fn from_mod(module: syn::ItemMod) -> Self {
        let name = module.ident;
        let mut items = module.content.unwrap().1.into_iter();
        let ty =
            *get_variant!(syn::Item::Type, items.next().unwrap()).unwrap().ty;
        let members = items.map(|member| {
            let item_const = get_variant!(syn::Item::Const, member)
                .unwrap();
            let name = item_const.ident;
            let expr = *item_const.expr;
            (name, expr)
        }).collect();
        let mut result = Enum { name, ty, members };
        result.rewrite_idents();
        result
    }

    fn rewrite_idents(&mut self) {
        let prefix = enum_prefix(strip_vendor_suffix(&self.name.to_string()));
        for member in self.members.iter_mut() {
            let ident = &mut member.0;
            let name = ident.to_string();
            let mut new_name = match strip_prefix(&name, &prefix) {
                Some(rest) => rest.to_string(),
                None => continue,
            };
            // E.g. VK_IMAGE_VIEW_TYPE_1D
            if new_name.chars().next().unwrap().is_digit(10)
                { new_name.insert(0, '_'); }
            *ident = syn::Ident::new(&new_name, ident.span());
        }
    }
}

// One problem with bindgen is that, if you use module-based enums, it
// replaces all occurrences of <enum name> with <enum name>::Type. Since
// we turn these modules into structs, we have to fix this.
// (though adding an associated type is a future possiblity)
struct EnumTypeVisitor;

impl syn::visit_mut::VisitMut for EnumTypeVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        let segs = &mut path.segments;
        if segs.len() >= 2
            && segs.last().unwrap().into_value().ident.to_string() == "Type"
        {
            segs.pop();
            let seg = segs.pop().unwrap().into_value();
            segs.push(seg);
        }
        syn::visit_mut::visit_path_mut(self, path);
    }
}

fn bitmask_traits(name: &syn::Ident) -> syn::File {
    // Sorry if this horrible ugliness offends you
    parse_quote! {
        impl ::std::ops::BitAnd for #name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output
                { #name(self.0.bitand(rhs.0)) }
        }
        impl ::std::ops::BitAndAssign for #name {
            fn bitand_assign(&mut self, rhs: Self)
                { self.0.bitand_assign(rhs.0); }
        }
        impl ::std::ops::BitOr for #name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output
                { #name(self.0.bitor(rhs.0)) }
        }
        impl ::std::ops::BitOrAssign for #name {
            fn bitor_assign(&mut self, rhs: Self)
                { self.0.bitor_assign(rhs.0); }
        }
        impl ::std::ops::BitXor for #name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output
                { #name(self.0.bitxor(rhs.0)) }
        }
        impl ::std::ops::BitXorAssign for #name {
            fn bitxor_assign(&mut self, rhs: Self)
                { self.0.bitxor_assign(rhs.0); }
        }
        impl ::std::ops::Not for #name {
            type Output = Self;
            fn not(self) -> Self::Output
                { #name(self.0.not()) }
        }
        impl #name {
            pub fn empty() -> Self { #name(0) }
            pub fn is_empty(self) -> bool { self.0 == 0 }
            pub fn intersects(self, other: Self) -> bool
                { self.0.bitand(other.0) > 0 }
            pub fn contains(self, other: Self) -> bool
                { self.0.bitand(other.0) == other.0 }
        }
    }
}

fn emit_enum(items: &mut Vec<syn::Item>, def: Enum) {
    let is_bitmask = def.is_bitmask();
    let name = def.name;
    let ty = def.ty;
    let members = def.members.into_iter().map(|(mem_name, expr)| {
        let decl: syn::ImplItemConst = parse_quote! {
            pub const #mem_name: #name = #name(#expr);
        };
        decl
    });
    let struct_def: syn::File = parse_quote! {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct #name(pub #ty);
        impl #name { #(#members)* }
    };
    items.extend(struct_def.items);
    if is_bitmask { items.extend(bitmask_traits(&name).items); }
}

// Parses a `use` statement of the form `use A::B::C as D`.
fn parse_use_tree(mut tree: syn::UseTree) -> (Vec<syn::Ident>, syn::Ident) {
    let mut idents = Vec::new();
    let rename = loop { match tree {
        syn::UseTree::Path(syn::UsePath { ident, tree: subtree, .. }) =>
            { idents.push(ident); tree = *subtree; },
        syn::UseTree::Rename(syn::UseRename { ident, rename, .. }) =>
            { idents.push(ident); break rename; },
        _ => panic!("Unexpected use statement"),
    } };
    (idents, rename)
}

// Bindgen emits aliases as `use self::<enum name>::Type`.
fn rewrite_alias(items: &mut Vec<syn::Item>, tree: syn::UseTree) {
    let (mut idents, rename) = parse_use_tree(tree);
    assert_eq!(idents.pop().unwrap().to_string(), "Type");
    let orig_name = idents.pop().unwrap();
    assert_eq!(idents.pop().unwrap().to_string(), "self");
    let new_item: syn::Item = parse_quote! {
        pub type #rename = #orig_name;
    };
    items.push(new_item);
}

crate fn do_rewrite(ast: &mut syn::File) {
    syn::visit_mut::visit_file_mut(&mut EnumTypeVisitor, ast);
    let items = std::mem::replace(&mut ast.items, Default::default());
    for item in items.into_iter() {
        match item {
            syn::Item::Mod(inner) => {
                let def = Enum::from_mod(inner);
                emit_enum(&mut ast.items, def);
            },
            syn::Item::Use(inner) => rewrite_alias(&mut ast.items, inner.tree),
            other => { ast.items.push(other); continue; },
        };
    }
}
