use heck::*;

use super::{map_ident, strip_prefix, strip_suffix};
use super::defs::*;

// TODO: This needs to be pulled from the registry
const VENDOR_TAGS: &[&'static str] = &[
    "Img", "Amd", "Amdx", "Arm", "Fsl", "Brcm", "Nxp", "Nv", "Nvx", "Viv",
    "Vsi", "Kdab", "Android", "Chromium", "Fuchsia", "Google", "Qcom",
    "Lunarg", "Samsung", "Sec", "Tizen", "Renderdoc", "Nn", "Mvk", "Khr",
    "Khx", "Ext", "Mesa",
];

// ** Enums **

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

fn rewrite_enum_idents(enum_: &mut Enum) {
    let prefix = enum_prefix(strip_vendor_suffix(&enum_.name.to_string()));
    for member in enum_.members.iter_mut() {
        let ident = &mut member.name;
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

fn parse_enum(module: syn::ItemMod) -> Enum {
    let name = module.ident;
    let mut items = module.content.unwrap().1.into_iter();
    let ty = get_variant!(syn::Item::Type, items.next().unwrap()).unwrap().ty;
    let members = items.map(|member| {
        let item_const = get_variant!(syn::Item::Const, member).unwrap();
        let name = item_const.ident;
        let value = item_const.expr;
        EnumMember { name, value }
    }).collect();
    let mut result = Enum { name, ty, members };
    rewrite_enum_idents(&mut result);
    result
}

// Bindgen emits enum aliases as `use self::<enum name>::Type`.
fn parse_enum_alias(tree: syn::UseTree) -> TypeAlias {
    let (mut idents, rename) = parse_use_tree(tree);
    assert_eq!(idents.pop().unwrap().to_string(), "Type");
    let target = idents.pop().unwrap();
    let target = parse_quote!(#target);
    assert_eq!(idents.pop().unwrap().to_string(), "self");
    let name = rename;
    TypeAlias { name, target }
}

// ** Constants **

fn parse_const(decl: syn::ItemConst) -> Const {
    let name = decl.ident;
    let ty = decl.ty;
    let value = decl.expr;
    Const { name, ty, value }
}

// ** Structs and unions **

fn parse_struct_member(field: syn::Field) -> StructMember {
    let name = field.ident.unwrap();
    let ty = field.ty;
    StructMember { name, ty }
}

fn parse_struct_members(fields: syn::FieldsNamed) -> Vec<StructMember> {
    fields.named.into_iter().map(parse_struct_member).collect()
}

fn parse_struct(decl: syn::ItemStruct) -> Struct {
    let name = decl.ident;
    let fields = get_variant!(syn::Fields::Named, decl.fields).unwrap();
    let members = parse_struct_members(fields);
    Struct { name, members }
}

fn parse_union(decl: syn::ItemUnion) -> Union {
    let name = decl.ident;
    let members = parse_struct_members(decl.fields);
    Union(Struct { name, members })
}

// ** Function pointers **

fn parse_function_pointer(name: syn::Ident, mut path: syn::Path) -> FnPointer
{
    let base_name =
        map_ident(&name, |s| strip_prefix(&s, "PFN_vk").unwrap().to_string());

    // bindgen wraps function pointers in `Option` to make them nullable
    let last_seg = path.segments.pop().unwrap().into_value();
    assert_eq!(last_seg.ident.to_string(), "Option");
    let args = last_seg.arguments;
    let mut args = get_variant!(syn::PathArguments::AngleBracketed, args)
        .unwrap().args;
    assert_eq!(args.len(), 1);
    let arg = args.pop().unwrap().into_value();
    let ty = get_variant!(syn::GenericArgument::Type, arg).unwrap();
    let signature = get_variant!(syn::Type::BareFn, ty).unwrap();

    FnPointer { base_name, signature }
}

// ** Top-level parsing **

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

fn parse_type_decl(decl: syn::ItemType) -> Def {
    let name = decl.ident;
    match decl.ty {
        box syn::Type::Ptr(_) =>
            Handle { name, dispatchable: true }.into(),
        box syn::Type::Path(syn::TypePath { path, .. }) => {
            let last_ident = &path.segments.last().unwrap().into_value().ident;
            let last_name = last_ident.to_string();
            match &last_name[..] {
                // This here is a magic word
                "NonDispatchableHandleVkFfi" =>
                    Handle { name, dispatchable: false }.into(),
                "Option" => parse_function_pointer(name, path).into(),
                "Flags" => {
                    let target_name = name.to_string()
                        .replace("Flags", "FlagBits");
                    let target =
                        syn::Ident::new(&target_name, last_ident.span());
                    let target = parse_quote!(#target);
                    TypeAlias { name, target }.into()
                },
                _ => TypeAlias { name, target: path }.into(),
            }
        },
        _ => panic!("Unexpected type alias"),
    }
}

fn parse_item(item: syn::Item) -> Option<Def> {
    Some(match item {
        syn::Item::Mod(module) => parse_enum(module).into(),
        syn::Item::Const(decl) => parse_const(decl).into(),
        syn::Item::Struct(decl) => {
            // We map "YYY_T" to `extern "C" type YYY`,
            // which really is bindgen's job but w/e
            if decl.ident.to_string().ends_with('T') { return None; }
            else { parse_struct(decl).into() }
        },
        syn::Item::Union(decl) => parse_union(decl).into(),
        syn::Item::Use(decl) => parse_enum_alias(decl.tree).into(),
        syn::Item::Type(decl) => parse_type_decl(decl).into(),
        _ => panic!("Unexpected item"),
    })
}

// This hack addresses the fact that many "*Flags" type alias are
// placeholders with no corresponding "*FlagBits" definition.
// We keep things uniform by adding an empty "*FlagBits" enum.
fn add_placeholder_enums(defs: &mut Defs) {
    use std::collections::HashSet;
    let known_enums: HashSet<_> = defs.enums.iter()
        .map(|enum_| enum_.name.to_string())
        .collect();
    for type_alias in defs.type_aliases.iter() {
        if type_alias.target.segments.len() > 1 { continue; }
        let target_ident = &type_alias.target.segments.last().unwrap()
            .into_value().ident;
        let target_name = target_ident.to_string();
        if !target_name.contains("FlagBits") { continue; }
        if !known_enums.contains(&target_name) {
            defs.enums.push(Enum {
                name: syn::Ident::new(&target_name, target_ident.span()),
                ty: parse_quote!(u32),
                members: Vec::new(),
            });
        }
    }
}

crate fn parse_file(ast: syn::File) -> Defs {
    let mut defs: Defs = Default::default();
    for item in ast.items.into_iter() {
        match parse_item(item) {
            Some(Def::Enum(val)) => defs.enums.push(val),
            Some(Def::Const(val)) => defs.consts.push(val),
            Some(Def::Struct(val)) => defs.structs.push(val),
            Some(Def::Union(val)) => defs.unions.push(val),
            Some(Def::FnPointer(val)) => defs.fn_pointers.push(val),
            Some(Def::TypeAlias(val)) => defs.type_aliases.push(val),
            Some(Def::Handle(val)) => defs.handles.push(val),
            None => continue,
        }
    }
    add_placeholder_enums(&mut defs);
    defs
}
