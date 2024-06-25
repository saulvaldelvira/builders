use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::util::{self, get_stripped_generics};

pub (crate) fn constructor_derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let vis = &ast.vis;
    let name = &ast.ident;

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

    let generics = &ast.generics;
    let wher = &generics.where_clause;
    let stripped_generics = get_stripped_generics(generics);

    quote! {
        impl #generics #name #stripped_generics #wher {
            #vis fn new(
                #( #args ,)*
            ) -> #name #stripped_generics {
                Self {
                    #( #names ,)*
                }
            }
        }

    }.into()
}
