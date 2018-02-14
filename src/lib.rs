//! 实现 `#[inherit_from()]` macro
//! [Cherry][sp]
//!
//! [sp]: https://dailypips.github.io/cherry-website/

#![feature(proc_macro)]
extern crate proc_macro;

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::{ToTokens, Tokens};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Fields, Ident, Type};

struct Attrs {
    types: Vec<Ident>, //error must be type
}

impl Synom for Attrs {
    named!(parse -> Self, map!(
        parens!(Punctuated::<Ident, Token![,]>::parse_separated),
        |(_parens, vars)| Attrs {
            types: vars.into_iter().collect(),
        }
    ));
}

fn get_first_field_type_ident(ast : &syn::ItemStruct) -> Option<Ident>{
    match ast.fields {
        Fields::Named(ref fields) => {
            let first = fields.named.iter().next().unwrap().clone();
            match first.ty {
                Type::Path(type_path) => {
                    let segment = type_path.path.segments.iter().last().unwrap();
                    return Some(segment.ident);
                },
                _=> (),
            };
        },
        _=> ()
    }
    None
}

fn check_first_field_type(types: &Vec<Ident>, ast : &syn::ItemStruct) -> bool
{
    let is_root_class = types.is_empty();
    let ident = get_first_field_type_ident(&ast).unwrap();

    if !is_root_class {
        let last_derived_from = types.iter().last().unwrap();

        if !(last_derived_from.to_string() == ident.to_string()) {
            return false
        }
    }
    true
}

#[proc_macro_attribute]
pub fn inherit_from(args: TokenStream, input: TokenStream) -> TokenStream {
    let struct_node: syn::ItemStruct = match syn::parse(input) {
        Ok(input) => input,
        Err(_) => panic!("`#[inherit_from] only apply to struct"),
    };

    let attrs: Attrs = match syn::parse(args) {
        Ok(input) => input,
        Err(_) => panic!("`#[inherit_from] without params"),
    };

    let check = check_first_field_type(&attrs.types, &struct_node);

    if !check {
        panic!("first field in struct not derivedFrom {}", attrs.types.iter().last().unwrap());
    }

    let mut tokens = Tokens::new();

    struct_node.to_tokens(&mut tokens);

    let struct_name = struct_node.ident;

    let (impl_generics, ty_generics, where_clause) = struct_node.generics.split_for_impl();

    tokens.append_all(quote! {
        impl #impl_generics Castable for #struct_name #ty_generics #where_clause {}
    });



    tokens.append_all(attrs.types.iter().map(|superclass| {
        quote! {
                    impl #impl_generics DerivedFrom<#superclass> for #struct_name #ty_generics #where_clause {}
        }
    }));

    tokens.to_string().parse().unwrap()
}
