// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Data, DataStruct, Fields, Ident, Token, Visibility,
};

// Procedural Macros: A simple derive macro
// Published 2021-02-20 on blog.turbo.fish
// https://blog.turbo.fish/proc-macro-simple-derive/

pub fn expand_getters(input: TokenStream) -> syn::Result<TokenStream2> {
    let input: syn::DeriveInput = syn::parse(input).expect("Couldn't parse item");

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let getters = fields
        .into_iter()
        .map(|f| {
            let meta: GetterMetaData = f
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("getter"))
                .try_fold(GetterMetaData::default(), |meta, attr| {
                    let list: Punctuated<GetterMetaData, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;
                    list.into_iter().try_fold(meta, GetterMetaData::merge)
                })?;
            let visibility = meta.vis.unwrap_or_else(|| parse_quote! { pub });
            let method_name = meta
                .name
                .unwrap_or_else(|| f.ident.clone().expect("a named field"));
            let field_name = f.ident;
            let field_ty = f.ty;

            Ok(quote! {
                #visibility fn #method_name(&self) -> &#field_ty {
                    &self.#field_name
                }
            })
        })
        .collect::<syn::Result<TokenStream2>>()?;

    let st_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #st_name #ty_generics #where_clause {
            #getters
        }
    })
}

#[derive(Default)]
struct GetterMetaData {
    name: Option<Ident>,
    vis: Option<Visibility>,
}

impl GetterMetaData {
    fn merge(self, other: GetterMetaData) -> syn::Result<Self> {
        fn either<T: ToTokens>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(a), Some(b)) => {
                    let mut error = syn::Error::new_spanned(a, "redundant attribute argument");
                    error.combine(syn::Error::new_spanned(b, "note: first one here"));
                    Err(error)
                }
            }
        }

        Ok(Self {
            name: either(self.name, other.name)?,
            vis: either(self.vis, other.vis)?,
        })
    }
}

impl Parse for GetterMetaData {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            let _: Token![=] = input.parse()?;
            let name = input.parse()?;
            Ok(Self {
                name: Some(name),
                vis: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(name);
    custom_keyword!(vis);
}
