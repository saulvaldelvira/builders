use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::util;

pub (crate) fn constructor_derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let vis = &ast.vis;
    let name = &ast.ident;
    let generics = &ast.generics;

    let fields = match util::get_named_struct(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error().into(),
    };

    let args = fields.iter().map(|f| {
        let ident = util::get_field_ident(f);
        let ty = &f.ty;
        quote! {
            #ident : impl std::convert::Into<#ty>
        }
    });
    let names = fields.iter().map(|f| {
        let ident = util::get_field_ident(f);
        quote! {
            #ident : #ident.into()
        }
    });

    quote! {
        impl #generics #name #generics {
            #vis fn new(
                #( #args ,)*
            ) -> Self {
                Self {
                    #( #names ,)*
                }
            }
        }

    }.into()
}
