extern crate vk_ffi;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadError(pub &'static str);

impl From<&'static str> for LoadError {
    fn from(val: &'static str) -> Self { LoadError(val) }
}

impl From<LoadError> for &'static str {
    fn from(val: LoadError) -> Self { val.0 }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to load function '{}'", self.0)
    }
}

impl std::error::Error for LoadError {}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/loader.rs"));
