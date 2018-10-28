use syn::Ident;

macro_rules! get_variant {
    ($var:path, $val:expr) => {
        if let $var(inner) = $val { Some(inner) } else { None }
    }
}

crate mod enum_rewrite;
crate mod global_rename;
crate mod handle_rewrite;

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

fn map_ident<A: AsRef<str>>(f: impl FnOnce(String) -> A, ident: Ident) ->
    Ident
{
    Ident::new(f(ident.to_string()).as_ref(), ident.span())
}
