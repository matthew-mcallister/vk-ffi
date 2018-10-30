use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;
use syn::punctuated::Punctuated;

use crate::*;

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        let members = &self.members;
        tokens.extend(quote!(#name: #ty { #(#members,)* }));
    }
}

impl Parse for Enum {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let content;
        braced!(content in input);
        let members_punct: Punctuated<EnumMember, Token![,]> =
            content.parse_terminated(EnumMember::parse)?;
        let members = members_punct.into_iter().collect();
        Ok(Enum { name, ty, members })
    }
}

impl ToTokens for EnumMember {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;
        tokens.extend(quote!(#name = #value));
    }
}

impl Parse for EnumMember {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(EnumMember { name, value })
    }
}

impl ToTokens for Const {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        let value = &self.value;
        tokens.extend(quote!(#name: #ty = #value));
    }
}

impl Parse for Const {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Const { name, ty, value })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let members = &self.members;
        tokens.extend(quote!(#name { #(#members,)* }));
    }
}

impl Parse for Struct {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        let content;
        braced!(content in input);
        let members_punct: Punctuated<StructMember, Token![,]> =
            content.parse_terminated(StructMember::parse)?;
        let members = members_punct.into_iter().collect();
        Ok(Struct { name, members })
    }
}

impl ToTokens for StructMember {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote!(#name: #ty));
    }
}

impl Parse for StructMember {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(StructMember { name, ty })
    }
}

impl ToTokens for Union {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl Parse for Union {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(Union(Struct::parse(input)?))
    }
}

impl ToTokens for FnPointer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let base_name = &self.base_name;
        let signature = &self.signature;
        tokens.extend(quote!(#base_name = #signature));
    }
}

impl Parse for FnPointer {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let base_name = input.parse()?;
        input.parse::<Token![=]>()?;
        let signature = input.parse()?;
        Ok(FnPointer { base_name, signature })
    }
}

impl ToTokens for TypeAlias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let target = &self.target;
        tokens.extend(quote!(#name = #target));
    }
}

impl Parse for TypeAlias {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let target = input.parse()?;
        Ok(TypeAlias { name, target })
    }
}

impl ToTokens for Handle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let dispatchable = self.dispatchable;
        tokens.extend(quote!(#name #dispatchable));
    }
}

impl Parse for Handle {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let dispatch_qual: Option<syn::Ident> = input.parse()?;
        let dispatchable = if let Some(ident) = dispatch_qual {
            match &ident.to_string()[..] {
                "dispatchable" => true,
                _ => return Err(syn::parse::Error::new
                    (ident.span(), "expected `dispatchable`")),
            }
        } else { false };
        Ok(Handle { name, dispatchable })
    }
}
