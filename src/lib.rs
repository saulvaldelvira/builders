mod util;
#[cfg(feature = "builder")]
mod builder;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use syn::{Meta, MetaList};

#[cfg(feature = "builder")]
#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    builder::builder_derive_impl(input)
}


#[cfg(feature = "getters")]
#[proc_macro_derive(Getters, attributes(getter,getters))]
pub fn getters_derive(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{parse_macro_input, DeriveInput};
    let ast = parse_macro_input!(input as DeriveInput);

    let prefix = util::get_prefix(&ast.attrs, "get_", "getters");

    util::gen_for_each_field(&ast, "getter", &prefix, |field,_each| {
        let ident = util::get_field_ident(field);
        let ty = &field.ty;
        /* if each { */
        /*     if let Some(v) = util::get_inner_ty(ty) { */
        /*         if v.las */
        /*     } */

        /*     if let Some((out,inner)) = util::get_inner_tys(ty,&["Vec","HashMap"]) { */
        /*         let first = inner[0]; */
        /*         return match out { */
        /*             "Vec" => { */
        /*                 quote! { */
        /*                     (&self, i: usize) -> ::core::option::Option<&#first> { */
        /*                         self.#ident.get(i) */
        /*                     } */
        /*                 } */
        /*             }, */
        /*             "HashMap" => { */
        /*                 let second = inner[1]; */
        /*                 quote! { */
        /*                     (&self, key: & #first) -> ::core::option::Option<&#second> { */
        /*                         self.#ident.get(key) */
        /*                     } */
        /*                 } */
        /*             }, */
        /*             _ => unreachable!() */
        /*         } */
        /*     } */
        /* } */
        quote! {
            (&self) -> & #ty {
                & self.#ident
            }
        }
    })
}

#[cfg(feature = "setters")]
#[proc_macro_derive(Setters, attributes(setter,setters))]
pub fn setters_derive(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, DeriveInput};

    let ast = parse_macro_input!(input as DeriveInput);
    let prefix = util::get_prefix(&ast.attrs, "set_", "setters");

    util::gen_for_each_field(&ast, "setter", &prefix, |field,_each| {
        let ident = util::get_field_ident(field);
        let ty = &field.ty;
        quote::quote! {
            (&mut self, #ident: impl ::core::convert::Into<#ty>) {
                self.#ident = #ident.into();
            }
        }
    })
}

#[cfg(feature = "constructor")]
mod constructor;

#[cfg(feature = "constructor")]
#[proc_macro_derive(Constructor, attributes(constructor))]
pub fn constructor_derive(input: TokenStream) -> TokenStream {
    constructor::constructor_derive_impl(input)
}


#[cfg(feature = "as_box")]
#[proc_macro_derive(AsBox)]
pub fn auto_box_derive(input: TokenStream) -> TokenStream {
    use syn::parse_macro_input;

    let ast = parse_macro_input!(input as syn::DeriveInput);
    let ident = &ast.ident;
    let vis = &ast.vis;
    let generics = &ast.generics;
    let stripped = util::get_stripped_generics(generics);
    let wher = &generics.where_clause;

    quote::quote! {
        extern crate alloc;

        impl #generics #ident #stripped #wher {
            #vis fn as_box(self) -> ::alloc::boxed::Box<#ident #stripped> {
                std::boxed::Box::new(self)
            }
        }
    }.into()
}

#[cfg(feature = "into_enum")]
#[proc_macro_derive(IntoEnum, attributes(into_enum))]
pub fn wrap_enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = &ast.ident;
    let generics = &ast.generics;
    let stripped = util::get_stripped_generics(generics);
    let wher = &generics.where_clause;

    let mut enum_name = None;
    let mut field_name = ident.clone();

    for attr in ast.attrs {
        if let Meta::List(MetaList { ref tokens, .. }) = attr.meta {
            let mut tokens = tokens.clone().into_iter();

            loop {
                let Some(key) = tokens.next() else { break };
                let TokenTree::Ident(ref name) = key else { panic!("Expected ident, found {key:#?}") };
                match tokens.next().unwrap() {
                    TokenTree::Punct(ref p) => assert_eq!(p.as_char(), '='),
                    tt => panic!("Expected '=', found: {tt}"),
                }
                let val = tokens.next().unwrap();
                let TokenTree::Ident(ref val) = val else { panic!("Expected ident, found {val:#?}") };

                if name == "enum_name" {
                    enum_name = Some(val.clone());
                }
                if name == "field" {
                    field_name = val.clone();
                }

                if let Some(key) = tokens.next() {
                    let TokenTree::Punct(ref name) = key else { panic!("Expected ',', found {key:#?}") };
                    if name.as_char() == ',' {
                        continue;
                    } else {
                        panic!()
                    }
                }
            }
        }
    }

    let enum_name = enum_name.unwrap();

    quote::quote! {
        impl #generics From<#ident #stripped> for #enum_name #wher {
            fn from(value: #ident #stripped) -> Self {
                #enum_name :: #field_name (value)
            }
        }
    }.into()
}
