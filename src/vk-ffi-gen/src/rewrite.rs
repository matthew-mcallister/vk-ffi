use heck::*;
use proc_macro2::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

use super::strip_prefix;

// TODO: map_* should return `impl ToTokens` or similar
trait MapTokens {
    fn map_ident(&mut self, ident: Ident) -> Ident { ident }
    fn map_punct(&mut self, punct: Punct) -> Punct { punct }
    fn map_literal(&mut self, literal: Literal) -> Literal { literal }
}

fn map_token_stream(
    map: &mut (impl MapTokens + ?Sized),
    tokens: TokenStream,
) -> TokenStream {
    tokens.into_iter().map(|tt| map_token_tree(map, tt)).collect()
}

fn map_token_tree(
    map: &mut (impl MapTokens + ?Sized),
    tt: TokenTree,
) -> TokenTree {
    match tt {
        TokenTree::Group(group) => {
            let delim = group.delimiter();
            let tokens = group.stream();
            let tokens = map_token_stream(map, tokens);
            let mut group_ = Group::new(delim, tokens);
            group_.set_span(group.span());
            TokenTree::Group(group_)
        },
        TokenTree::Ident(ident) => TokenTree::Ident(map.map_ident(ident)),
        TokenTree::Punct(punct) => TokenTree::Punct(map.map_punct(punct)),
        TokenTree::Literal(lit) => TokenTree::Literal(map.map_literal(lit)),
    }
}

struct RenameMap;

const MIXED_PREFIX: &'static str = "vk";
const CAMEL_PREFIX: &'static str = "Vk";
const SHOUTY_PREFIX: &'static str = "VK_";
const PFN_PREFIX: &'static str = "PFN_vk";

// Bindgen renames things that conflict with keywords by appending "_".
// Tests whether an ident matches this pattern so we don't undo it.
fn is_bindgen_renamed_keyword(ident: &str) -> bool {
    // Heuristic implementation: nothing in the Vulkan API ends with "_"
    ident.ends_with("_")
}

fn heuristic_rename(ident: &str) -> Option<String> {
    if let Some(stripped) = strip_prefix(ident, MIXED_PREFIX) {
        Some(stripped.to_snake_case())
    } else if let Some(stripped) = strip_prefix(ident, CAMEL_PREFIX) {
        Some(stripped.to_camel_case())
    } else if let Some(stripped) = strip_prefix(ident, SHOUTY_PREFIX) {
        Some(stripped.to_shouty_snake_case())
    } else if ident.starts_with(PFN_PREFIX) {
        // These identifiers are deferred until AFTER code generation
        None
    } else if ident.chars().next().unwrap().is_uppercase() {
        Some(ident.to_camel_case())
    } else if ident.chars().next().unwrap().is_lowercase()
        && !is_bindgen_renamed_keyword(ident)
    {
        Some(ident.to_snake_case())
    } else { None }
}

impl MapTokens for RenameMap {
    fn map_ident(&mut self, ident: Ident) -> Ident {
        let res: Option<Ident> = try {
            let orig = ident.to_string();
            let new = heuristic_rename(&orig)?;
            Ident::new(&new, ident.span())
        };
        res.unwrap_or(ident)
    }
}

crate fn rewrite_tokens(tokens: TokenStream) -> TokenStream {
    map_token_stream(&mut RenameMap, tokens)
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

// Get rid of __int8_t etc.
fn skip_compiler_types(ast: &mut syn::File) {
    ast.items.retain(|item| {
        if let syn::Item::Type(ref ty_def) = item {
            !ty_def.ident.to_string().starts_with("__")
        } else { true }
    });
}

crate fn rewrite_ast(ast: &mut syn::File) {
    skip_compiler_types(ast);
    syn::visit_mut::visit_file_mut(&mut EnumTypeVisitor, ast);
}

// We have to go through identifier renaming again to replace any
// remaining occurrences of "PFN_vk*".
struct RenamePostMap;

fn rename_pfn(ident: &str) -> Option<String> {
    let stripped = strip_prefix(ident, PFN_PREFIX)?;
    Some(format!("Pfn{}", stripped.to_camel_case()))
}

impl MapTokens for RenamePostMap {
    fn map_ident(&mut self, ident: Ident) -> Ident {
        let res: Option<Ident> = try {
            let orig = ident.to_string();
            let new = rename_pfn(&orig)?;
            Ident::new(&new, ident.span())
        };
        res.unwrap_or(ident)
    }
}

crate fn rewrite_tokens_post(tokens: TokenStream) -> TokenStream {
    map_token_stream(&mut RenamePostMap, tokens)
}
