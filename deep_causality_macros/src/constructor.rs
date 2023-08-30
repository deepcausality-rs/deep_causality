// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{punctuated::Punctuated, Token};

pub fn expand_constructor(input: TokenStream) -> syn::Result<TokenStream2> {
    let ast: syn::DeriveInput = syn::parse(input).expect("Couldn't parse item");
    let result = match ast.data {
        syn::Data::Enum(ref e) => new_for_enum(&ast, e),
        syn::Data::Struct(ref s) => new_for_struct(&ast, &s.fields, None),
        syn::Data::Union(_) => panic!("doesn't work with unions yet"),
    };
    Ok(result)
}

macro_rules! my_quote {
    ($($t:tt)*) => (quote_spanned!(proc_macro2::Span::call_site() => $($t)*))
}

fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

fn new_for_struct(
    ast: &syn::DeriveInput,
    fields: &syn::Fields,
    variant: Option<&syn::Ident>,
) -> proc_macro2::TokenStream {
    match *fields {
        syn::Fields::Named(ref fields) => new_impl(ast, Some(&fields.named), true, variant),
        syn::Fields::Unit => new_impl(ast, None, false, variant),
        syn::Fields::Unnamed(ref fields) => new_impl(ast, Some(&fields.unnamed), false, variant),
    }
}

fn new_for_enum(ast: &syn::DeriveInput, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    if data.variants.is_empty() {
        panic!("#[derive(new)] cannot be implemented for enums with zero variants");
    }
    let impls = data.variants.iter().map(|v| {
        if v.discriminant.is_some() {
            panic!("#[derive(new)] cannot be implemented for enums with discriminants");
        }
        new_for_struct(ast, &v.fields, Some(&v.ident))
    });
    my_quote!(#(#impls)*)
}

fn new_impl(
    ast: &syn::DeriveInput,
    fields: Option<&Punctuated<syn::Field, Token![,]>>,
    named: bool,
    variant: Option<&syn::Ident>,
) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let unit = fields.is_none();
    let empty = Default::default();
    let fields: Vec<_> = fields
        .unwrap_or(&empty)
        .iter()
        .enumerate()
        .map(|(i, f)| FieldExt::new(f, i, named))
        .collect();
    let args = fields.iter().filter(|f| f.needs_arg()).map(|f| f.as_arg());
    let inits = fields.iter().map(|f| f.as_init());
    let inits = if unit {
        my_quote!()
    } else if named {
        my_quote![{ #(#inits),* }]
    } else {
        my_quote![( #(#inits),* )]
    };
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let (mut new, qual, doc) = match variant {
        None => (
            syn::Ident::new("new", proc_macro2::Span::call_site()),
            my_quote!(),
            format!("Constructs a new `{}`.", name),
        ),
        Some(ref variant) => (
            syn::Ident::new(
                &format!("new_{}", to_snake_case(&variant.to_string())),
                proc_macro2::Span::call_site(),
            ),
            my_quote!(::#variant),
            format!("Constructs a new `{}::{}`.", name, variant),
        ),
    };
    new.set_span(proc_macro2::Span::call_site());
    let lint_attrs = collect_parent_lint_attrs(&ast.attrs);
    let lint_attrs = my_quote![#(#lint_attrs),*];
    my_quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[doc = #doc]
            #lint_attrs
            pub fn #new(#(#args),*) -> Self {
                #name #qual #inits
            }
        }
    }
}

fn collect_parent_lint_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    fn is_lint(item: &syn::Meta) -> bool {
        if let syn::Meta::List(ref l) = *item {
            let path = &l.path;
            return path.is_ident("allow")
                || path.is_ident("deny")
                || path.is_ident("forbid")
                || path.is_ident("warn");
        }
        false
    }

    fn is_cfg_attr_lint(item: &syn::Meta) -> bool {
        if let syn::Meta::List(ref l) = *item {
            if l.path.is_ident("cfg_attr") {
                if let Ok(nested) =
                    l.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
                {
                    return nested.len() == 2 && is_lint(&nested[1]);
                }
            }
        }
        false
    }

    attrs
        .iter()
        .filter(|a| is_lint(&a.meta) || is_cfg_attr_lint(&a.meta))
        .cloned()
        .collect()
}

enum FieldAttr {
    Default,
    Value(proc_macro2::TokenStream),
}

impl FieldAttr {
    pub fn as_tokens(&self) -> proc_macro2::TokenStream {
        match *self {
            FieldAttr::Default => my_quote!(::core::default::Default::default()),
            FieldAttr::Value(ref s) => my_quote!(#s),
        }
    }

    pub fn parse(attrs: &[syn::Attribute]) -> Option<FieldAttr> {
        let mut result = None;
        for attr in attrs.iter() {
            match attr.style {
                syn::AttrStyle::Outer => {}
                _ => continue,
            }
            let last_attr_path = attr
                .path()
                .segments
                .last()
                .expect("Expected at least one segment where #[segment[::segment*](..)]");
            if (last_attr_path).ident != "new" {
                continue;
            }
            let list = match attr.meta {
                syn::Meta::List(ref l) => l,
                _ if attr.path().is_ident("new") => {
                    panic!("Invalid #[new] attribute, expected #[new(..)]")
                }
                _ => continue,
            };
            if result.is_some() {
                panic!("Expected at most one #[new] attribute");
            }
            for item in list
                .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
                .unwrap_or_else(|err| panic!("Invalid #[new] attribute: {}", err))
            {
                match item {
                    syn::Meta::Path(path) => {
                        if path.is_ident("default") {
                            result = Some(FieldAttr::Default);
                        } else {
                            panic!(
                                "Invalid #[new] attribute: #[new({})]",
                                path_to_string(&path)
                            );
                        }
                    }
                    syn::Meta::NameValue(kv) => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref s),
                            ..
                        }) = kv.value
                        {
                            if kv.path.is_ident("value") {
                                let tokens = lit_str_to_token_stream(s).unwrap_or_else(|_| {
                                    panic!("Invalid expression in #[new]: `{}`", s.value())
                                });
                                result = Some(FieldAttr::Value(tokens));
                            } else {
                                panic!(
                                    "Invalid #[new] attribute: #[new({} = ..)]",
                                    path_to_string(&kv.path)
                                );
                            }
                        } else {
                            panic!("Non-string literal value in #[new] attribute");
                        }
                    }
                    syn::Meta::List(l) => {
                        panic!(
                            "Invalid #[new] attribute: #[new({}(..))]",
                            path_to_string(&l.path)
                        );
                    }
                }
            }
        }
        result
    }
}

struct FieldExt<'a> {
    ty: &'a syn::Type,
    attr: Option<FieldAttr>,
    ident: syn::Ident,
    named: bool,
}

impl<'a> FieldExt<'a> {
    pub fn new(field: &'a syn::Field, idx: usize, named: bool) -> FieldExt<'a> {
        FieldExt {
            ty: &field.ty,
            attr: FieldAttr::parse(&field.attrs),
            ident: if named {
                field.ident.clone().unwrap()
            } else {
                syn::Ident::new(&format!("f{}", idx), proc_macro2::Span::call_site())
            },
            named,
        }
    }

    pub fn has_attr(&self) -> bool {
        self.attr.is_some()
    }

    pub fn is_phantom_data(&self) -> bool {
        match *self.ty {
            syn::Type::Path(syn::TypePath {
                qself: None,
                ref path,
            }) => path
                .segments
                .last()
                .map(|x| x.ident == "PhantomData")
                .unwrap_or(false),
            _ => false,
        }
    }

    pub fn needs_arg(&self) -> bool {
        !self.has_attr() && !self.is_phantom_data()
    }

    pub fn as_arg(&self) -> proc_macro2::TokenStream {
        let f_name = &self.ident;
        let ty = &self.ty;
        my_quote!(#f_name: #ty)
    }

    pub fn as_init(&self) -> proc_macro2::TokenStream {
        let f_name = &self.ident;
        let init = if self.is_phantom_data() {
            my_quote!(::core::marker::PhantomData)
        } else {
            match self.attr {
                None => my_quote!(#f_name),
                Some(ref attr) => attr.as_tokens(),
            }
        };
        if self.named {
            my_quote!(#f_name: #init)
        } else {
            my_quote!(#init)
        }
    }
}

fn lit_str_to_token_stream(s: &syn::LitStr) -> Result<TokenStream2, proc_macro2::LexError> {
    Ok(set_ts_span_recursive(s.value().parse()?, &s.span()))
}

fn set_ts_span_recursive(ts: TokenStream2, span: &proc_macro2::Span) -> TokenStream2 {
    ts.into_iter()
        .map(|mut tt| {
            tt.set_span(*span);
            if let proc_macro2::TokenTree::Group(group) = &mut tt {
                *group = proc_macro2::Group::new(
                    group.delimiter(),
                    set_ts_span_recursive(group.stream(), span),
                );
            }
            tt
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let (ch, next, mut acc): (Option<char>, Option<char>, String) =
        s.chars()
            .fold((None, None, String::new()), |(prev, ch, mut acc), next| {
                if let Some(ch) = ch {
                    if let Some(prev) = prev {
                        if ch.is_uppercase()
                            && (prev.is_lowercase()
                                || prev.is_numeric()
                                || (prev.is_uppercase() && next.is_lowercase()))
                        {
                            acc.push('_');
                        }
                    }
                    acc.extend(ch.to_lowercase());
                }
                (ch, Some(next), acc)
            });
    if let Some(next) = next {
        if let Some(ch) = ch {
            if (ch.is_lowercase() || ch.is_numeric()) && next.is_uppercase() {
                acc.push('_');
            }
        }
        acc.extend(next.to_lowercase());
    }
    acc
}
