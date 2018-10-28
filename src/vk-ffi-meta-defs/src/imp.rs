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

impl ToTokens for FunctionPointer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let base_name = &self.base_name;
        let signature = &self.signature;
        tokens.extend(quote!(#base_name = #signature));
    }
}

impl Parse for FunctionPointer {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let base_name = input.parse()?;
        input.parse::<Token![=]>()?;
        let signature = input.parse()?;
        Ok(FunctionPointer { base_name, signature })
    }
}

impl ToTokens for Alias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let target = &self.target;
        tokens.extend(quote!(#name = #target));
    }
}

impl Parse for Alias {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let target = input.parse()?;
        Ok(Alias { name, target })
    }
}

impl ToTokens for DispatchableHandle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        tokens.extend(quote!(#name));
    }
}

impl Parse for DispatchableHandle {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(DispatchableHandle { name: input.parse()? })
    }
}

impl ToTokens for NonDispatchableHandle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        tokens.extend(quote!(#name));
    }
}

impl Parse for NonDispatchableHandle {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(NonDispatchableHandle { name: input.parse()? })
    }
}
