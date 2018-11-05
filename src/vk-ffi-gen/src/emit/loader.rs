use std::collections::HashMap;
use std::ffi::CString;
use std::process::Command;

use heck::*;
use proc_macro2::TokenStream;
use syn::parse;

use crate::{ident, map_ident, strip_prefix};
use crate::defs::*;

const SCRIPT_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/bin/registry");

const VK_XML_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/Vulkan-Docs/xml/vk.xml");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApiLevel {
    Instance,
    Device,
}

impl ApiLevel {
    fn as_str(self) -> &'static str {
        match self {
            ApiLevel::Instance => "Instance",
            ApiLevel::Device => "Device",
        }
    }

    fn handle(self) -> syn::Ident {
        ident(match self {
            ApiLevel::Instance => "instance",
            ApiLevel::Device => "device",
        })
    }

    fn handle_ty(self) -> syn::Ident {
        ident(self.as_str())
    }

    fn get_proc_addr(self) -> syn::Ident {
        ident(match self {
            ApiLevel::Instance => "get_instance_proc_addr",
            ApiLevel::Device => "get_device_proc_addr",
        })
    }

    fn get_proc_addr_ty(self) -> syn::Ident {
        ident(match self {
            ApiLevel::Instance => "FnGetInstanceProcAddr",
            ApiLevel::Device => "FnGetDeviceProcAddr",
        })
    }
}

impl std::str::FromStr for ApiLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Instance" => Ok(ApiLevel::Instance),
            "Device" => Ok(ApiLevel::Device),
            _ => Err(()),
        }
    }
}

impl parse::Parse for ApiLevel {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let ident: syn::Ident = input.parse().unwrap();
        ident.to_string().parse().map_err(|_| parse::Error::new
            (ident.span(), "Expected 'Instance' or 'Device'"))
    }
}

#[derive(Clone, Debug)]
struct Api {
    name: syn::Ident,
    level: ApiLevel,
    commands: Vec<syn::Ident>,
}

impl parse::Parse for Api {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let name = input.parse().unwrap();
        input.parse::<Token![:]>().unwrap();
        let level = input.parse().unwrap();
        let mut commands: Vec<syn::Ident> = Vec::new();
        let content;
        braced!(content in input);
        while !content.is_empty() {
            commands.push(content.parse().unwrap());
            content.parse::<Token![,]>().unwrap();
        }
        Ok(Api { name, level, commands })
    }
}

fn rewrite_api_idents(api: &mut Api) {
    api.name = map_ident
        (&api.name, |s| strip_prefix(&s, "VK_").unwrap().to_camel_case());
    for command in api.commands.iter_mut() {
        *command = map_ident
            (command, |s| strip_prefix(&s, "vk").unwrap().to_camel_case());
    }
}

#[derive(Debug)]
struct Apis {
    instance_v1_0: Api,
    instance_v1_1: Api,
    device_v1_0: Api,
    device_v1_1: Api,
    extensions: Vec<Api>,
}

macro_rules! parse_apis_impl {
    ($($name:ident: $level:expr,)*) => {
        fn parse(input: parse::ParseStream) -> parse::Result<Self> {
            $(
                let $name: Api = input.parse().unwrap();
                assert_eq!(
                    &$name.name.to_string(),
                    concat!("VK_", stringify!($name)));
                assert_eq!($name.level, $level);
            )*
            let mut extensions: Vec<Api> = Vec::new();
            while !input.is_empty() { extensions.push(input.parse().unwrap()); }
            Ok(Apis { $($name,)* extensions })
        }
    }
}

impl parse::Parse for Apis {
    parse_apis_impl! {
        instance_v1_0: ApiLevel::Instance,
        instance_v1_1: ApiLevel::Instance,
        device_v1_0: ApiLevel::Device,
        device_v1_1: ApiLevel::Device,
    }
}

fn rewrite_apis_idents(apis: &mut Apis) {
    rewrite_api_idents(&mut apis.instance_v1_0);
    apis.instance_v1_0.name = ident("CoreInstance");
    rewrite_api_idents(&mut apis.instance_v1_1);
    apis.instance_v1_1.name = ident("CoreInstance");
    rewrite_api_idents(&mut apis.device_v1_0);
    apis.device_v1_0.name = ident("CoreDevice");
    rewrite_api_idents(&mut apis.device_v1_1);
    apis.device_v1_1.name = ident("CoreDevice");
    for extension in apis.extensions.iter_mut() {
        rewrite_api_idents(extension);
    }
}

fn get_apis() -> Apis {
    let output = Command::new(SCRIPT_PATH)
        .arg(VK_XML_PATH)
        .output().unwrap()
        .stdout;
    let output = String::from_utf8(output).unwrap();
    let mut apis: Apis = syn::parse_str(&output).unwrap();
    rewrite_apis_idents(&mut apis);
    apis
}

fn emit_table(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) -> TokenStream
{
    let api_name = &api.name;
    let mut table_body: TokenStream =
        if api.level == ApiLevel::Instance { quote!(pub instance: Instance,) }
        else { quote!(pub device: Device,) };
    for cmd_name in api.commands.iter() {
        let pfn = fn_ptrs[&cmd_name.to_string()];
        let member_name = pfn.snake_name();
        let ty = pfn.fn_name();
        table_body.extend(quote!(pub #member_name: #ty,));
    }
    quote! {
        #[derive(Clone, Copy)]
        pub struct #api_name { #table_body }
    }
}

fn emit_loader(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) -> TokenStream
{
    let handle = api.level.handle();
    let handle_ty = api.level.handle_ty();
    let get_proc_addr = api.level.get_proc_addr();
    let get_proc_addr_ty = api.level.get_proc_addr_ty();

    let api_name = &api.name;
    let mut loader_body = TokenStream::new();
    for cmd_name in api.commands.iter() {
        let pfn = fn_ptrs[&cmd_name.to_string()];
        let member_name = pfn.snake_name();
        let symbol_cstr = CString::new(pfn.symbol_name()).unwrap();
        let symbol_bytes = proc_macro2::Literal::byte_string
            (symbol_cstr.as_bytes_with_nul());
        let symbol = symbol_cstr.to_str().unwrap();
        loader_body.extend(quote! {
            #member_name: ::std::mem::transmute({
                #get_proc_addr(#handle, #symbol_bytes.as_ptr() as *const _)
                    .ok_or(LoadError(#symbol))?
            }),
        });
    }

    quote! {
        impl #api_name {
            pub unsafe fn load(
                #handle: #handle_ty,
                #get_proc_addr: #get_proc_addr_ty,
            ) -> ::std::result::Result<Self, LoadError> {
                Ok(#api_name {
                    #handle,
                    #loader_body
                })
            }
        }
    }
}

fn type_is(ty: &syn::Type, name: &str) -> bool {
    let res: Option<bool> = try {
        let segs = &get_variant_ref!(syn::Type::Path, ty).unwrap().path.segments;
        if segs.len() != 1 { return false; }
        &segs[0].ident.to_string() == name
    };
    res.unwrap_or(false)
}

#[derive(Clone, Debug)]
struct MethodInputs<'a> {
    inner: &'a syn::punctuated::Punctuated<syn::BareFnArg, Token![,]>,
    level: ApiLevel,
    // If true, automatically pass the instance/device handle on the
    // function pointer table as the first argument.
    takes_handle: bool,
}

impl<'a> MethodInputs<'a> {
    fn new(level: ApiLevel, pfn: &'a FnPointer) -> MethodInputs<'a> {
        let inner = &pfn.signature.inputs;
        let takes_handle = type_is(&inner[0].ty, level.as_str());
        MethodInputs { inner, level, takes_handle }
    }

    fn names(&self) -> impl Iterator<Item = &'a syn::Ident> {
        let mut iter = self.inner.iter();
        if self.takes_handle { iter.next(); }
        iter.map(|arg| get_variant_ref!(
            syn::BareFnArgName::Named,
            arg.name.as_ref().unwrap().0
        ).unwrap())
    }

    fn args(&self) -> impl Iterator<Item = &'a syn::BareFnArg> {
        let mut iter = self.inner.iter();
        if self.takes_handle { iter.next(); }
        iter
    }
}

fn emit_methods(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) ->
    TokenStream
{
    let api_name = &api.name;
    let mut methods_body = TokenStream::new();
    for cmd_name in api.commands.iter() {
        let pfn = fn_ptrs[&cmd_name.to_string()];
        let inputs = MethodInputs::new(api.level, &pfn);
        let output = &pfn.signature.output;

        let member_name = pfn.snake_name();
        let handle = api.level.handle();
        let args = inputs.args();
        let names = inputs.names();
        methods_body.extend(if inputs.takes_handle {
            quote! {
                pub unsafe fn #member_name(&self, #(#args,)*) #output {
                    (self.#member_name)(self.#handle, #(#names,)*)
                }
            }
        } else {
            quote! {
                pub unsafe fn #member_name(&self, #(#args,)*) #output {
                    (self.#member_name)(#(#names,)*)
                }
            }
        });
    }
    quote!(impl #api_name { #methods_body } )
}

fn emit_api(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) ->
    Option<TokenStream>
{
    // Not all extensions are currently supported directly, namely the
    // WSI-related ones. We'll have to skip those.
    for cmd in api.commands.iter() { fn_ptrs.get(&cmd.to_string())?; }

    let mut tokens = emit_table(api, fn_ptrs);
    tokens.extend(emit_loader(api, fn_ptrs));
    tokens.extend(emit_methods(api, fn_ptrs));
    Some(tokens)
}

crate fn emit(fn_ptrs: &[FnPointer]) -> TokenStream {
    let fn_ptr_table = fn_ptrs.iter()
        .map(|pfn| (pfn.base_name.to_string().to_camel_case(), pfn))
        .collect();
    let apis = get_apis();

    let mut result = TokenStream::new();

    let mut tokens = emit_api(&apis.instance_v1_0, &fn_ptr_table).unwrap();
    tokens.extend(emit_api(&apis.device_v1_0, &fn_ptr_table).unwrap());
    result.extend(quote! {
        pub mod v1_0 {
            use vk_ffi::*;
            use crate::LoadError;
            pub use crate::entry::v1_0::*;
            pub use crate::extensions::*;
            #tokens
        }
    });

    let mut tokens = emit_api(&apis.instance_v1_1, &fn_ptr_table).unwrap();
    tokens.extend(emit_api(&apis.device_v1_1, &fn_ptr_table).unwrap());
    result.extend(quote! {
        pub mod v1_1 {
            use vk_ffi::*;
            use crate::LoadError;
            pub use crate::entry::v1_1::*;
            pub use crate::extensions::*;
            #tokens
        }
    });

    let mut tokens = TokenStream::new();
    for ext in apis.extensions.iter() {
        if let Some(toks) = emit_api(ext, &fn_ptr_table) {
            tokens.extend(toks);
        }
    }
    result.extend(quote! {
        pub mod extensions {
            use vk_ffi::*;
            use crate::LoadError;
            #tokens
        }
    });

    result
}
