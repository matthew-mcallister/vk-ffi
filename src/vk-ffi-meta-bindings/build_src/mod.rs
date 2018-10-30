use vk_ffi_meta_defs::*;

macro_rules! get_variant {
    ($var:path, $val:expr) => {
        if let $var(inner) = $val { Some(inner) } else { None }
    }
}

crate mod emit;
crate mod rewrite;
crate mod scrape;

fn split_prefix<'a>(s: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    if s.starts_with(prefix) { Some(s.split_at(prefix.len())) }
    else { None }
}

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    Some(split_prefix(s, prefix)?.1)
}

fn split_suffix<'a>(s: &'a str, suffix: &str) -> Option<(&'a str, &'a str)> {
    if s.ends_with(suffix) { Some(s.split_at(s.len() - suffix.len())) }
    else { None }
}

fn strip_suffix<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    Some(split_suffix(s, suffix)?.0)
}

#[derive(Default)]
#[cfg_attr(feature = "syn-extra-traits", derive(Clone, Debug))]
crate struct Defs {
    pub(super) enums: Vec<Enum>,
    pub(super) consts: Vec<Const>,
    pub(super) structs: Vec<Struct>,
    pub(super) unions: Vec<Union>,
    pub(super) fn_pointers: Vec<FnPointer>,
    pub(super) type_aliases: Vec<TypeAlias>,
    pub(super) handles: Vec<Handle>,
}
