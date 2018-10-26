use heck::*;

use super::{strip_prefix, strip_suffix};

crate fn do_rewrite(ast: &mut syn::File) {
    syn::visit_mut::visit_file_mut(&mut EnumRenameVisitor, ast);
}

struct EnumRenameVisitor;

// TODO: This needs to be pulled from the registry
const VENDOR_TAGS: &[&'static str] = &[
    "Img", "Amd", "Amdx", "Arm", "Fsl", "Brcm", "Nxp", "Nv", "Nvx", "Viv",
    "Vsi", "Kdab", "Android", "Chromium", "Fuchsia", "Google", "Qcom",
    "Lunarg", "Samsung", "Sec", "Tizen", "Renderdoc", "Nn", "Mvk", "Khr",
    "Khx", "Ext", "Mesa",
];

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

impl syn::visit_mut::VisitMut for EnumRenameVisitor {
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        let mod_name = module.ident.to_string();
        let prefix = enum_prefix(strip_vendor_suffix(&mod_name));

        let (_, ref mut items) = module.content.as_mut().unwrap();
        for item in items.iter_mut() {
            let _: Option<()> = try {
                let decl =
                    if let syn::Item::Const(ref mut decl) = *item { decl }
                    else { continue };
                let name = decl.ident.to_string();
                let mut new_name = strip_prefix(&name, &prefix)?.to_string();
                // E.g. VK_IMAGE_VIEW_TYPE_1D
                if new_name.chars().next().unwrap().is_digit(10)
                    { new_name.insert(0, '_'); }
                decl.ident = syn::Ident::new(&new_name, decl.ident.span());
            };
        }
    }
}
