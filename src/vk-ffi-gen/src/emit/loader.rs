use std::collections::{HashSet, HashMap};
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
    commands: HashSet<syn::Ident>,
}

impl parse::Parse for Api {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let level = input.parse().unwrap();
        let mut commands: HashSet<syn::Ident> = HashSet::new();
        let content;
        braced!(content in input);
        while !content.is_empty() {
            let ident: syn::Ident = content.parse().unwrap();
            let ident = map_ident
                (&ident, |s| strip_prefix(&s, "vk").unwrap().to_camel_case());
            commands.insert(ident);
            content.parse::<Token![,]>().unwrap();
        }

        let name = ident(match level {
            ApiLevel::Instance => "InstanceTable",
            ApiLevel::Device => "DeviceTable",
        });

        Ok(Api { name, level, commands })
    }
}

#[derive(Debug)]
struct Apis {
    instance: Api,
    device: Api,
}

impl parse::Parse for Apis {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let instance: Api = input.parse().unwrap();
        assert_eq!(instance.level, ApiLevel::Instance);
        let device: Api = input.parse().unwrap();
        assert_eq!(device.level, ApiLevel::Device);
        Ok(Apis { instance, device })
    }
}

fn get_apis() -> Apis {
    let output = Command::new(SCRIPT_PATH)
        .arg(VK_XML_PATH)
        .output().unwrap()
        .stdout;
    let output = String::from_utf8(output).unwrap();
    syn::parse_str(&output).unwrap()
}

fn emit_table(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) -> TokenStream
{
    let api_name = &api.name;
    let mut table_body: TokenStream =
        if api.level == ApiLevel::Instance { quote!(pub instance: Instance,) }
        else { quote!(pub device: Device,) };
    for cmd_name in api.commands.iter() {
        let pfn = match fn_ptrs.get(&cmd_name.to_string()) {
            Some(pfn) => pfn,
            // Commands from unsupported extensions are not loaded.
            None => continue,
        };
        let member_name = pfn.snake_name();
        let ty = pfn.pfn_name();
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
        let pfn = match fn_ptrs.get(&cmd_name.to_string()) {
            Some(pfn) => pfn,
            None => continue,
        };
        let member_name = pfn.snake_name();
        let symbol_cstr = CString::new(pfn.symbol_name()).unwrap();
        let symbol_bytes = proc_macro2::Literal::byte_string
            (symbol_cstr.as_bytes_with_nul());
        loader_body.extend(quote! {
            #member_name: ::std::mem::transmute({
                #get_proc_addr(#handle, #symbol_bytes.as_ptr() as *const _)
            }),
        });
    }

    quote! {
        impl #api_name {
            pub unsafe fn load(
                #handle: #handle_ty,
                #get_proc_addr: #get_proc_addr_ty,
            ) -> Self {
                #api_name {
                    #handle,
                    #loader_body
                }
            }
        }
    }
}

fn type_is(ty: &syn::Type, name: &str) -> bool {
    let res: Option<bool> = try {
        let segs =
            &get_variant_ref!(syn::Type::Path, ty).unwrap().path.segments;
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
        let pfn = match fn_ptrs.get(&cmd_name.to_string()) {
            Some(pfn) => pfn,
            None => continue,
        };
        let inputs = MethodInputs::new(api.level, &pfn);
        let output = &pfn.signature.output;

        let member_name = pfn.snake_name();
        let fn_type = pfn.fn_name();
        let handle = api.level.handle();
        let args = inputs.args();
        let names = inputs.names();
        methods_body.extend(if inputs.takes_handle {
            quote! {
                pub unsafe fn #member_name(&self, #(#args,)*) #output {
                    std::mem::transmute::<_, #fn_type>(self.#member_name)
                        (self.#handle, #(#names,)*)
                }
            }
        } else {
            quote! {
                pub unsafe fn #member_name(&self, #(#args,)*) #output {
                    std::mem::transmute::<_, #fn_type>(self.#member_name)
                        (#(#names,)*)
                }
            }
        });
    }
    quote!(impl #api_name { #methods_body })
}

fn emit_api(api: &Api, fn_ptrs: &HashMap<String, &FnPointer>) -> TokenStream {
    let mut tokens = emit_table(api, fn_ptrs);
    tokens.extend(emit_loader(api, fn_ptrs));
    tokens.extend(emit_methods(api, fn_ptrs));
    tokens
}

crate fn emit(fn_ptrs: &[FnPointer]) -> TokenStream {
    let fn_ptr_table = fn_ptrs.iter()
        .map(|pfn| (pfn.base_name.to_string().to_camel_case(), pfn))
        .collect();
    let apis = get_apis();

    [&apis.instance, &apis.device].iter()
        .map(|&api| emit_api(api, &fn_ptr_table))
        .collect()
}
