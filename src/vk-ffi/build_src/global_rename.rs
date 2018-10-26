use heck::*;
use proc_macro2::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

use super::strip_prefix;

crate fn do_rename(tokens: TokenStream) -> TokenStream {
    map_token_stream(&mut RenameMap, tokens)
}

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
