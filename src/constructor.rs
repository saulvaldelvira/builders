use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Expr, ExprLit, Field, Lit, LitBool, Meta, MetaNameValue};

use crate::util::{self, get_stripped_generics};

fn must_generate(f: &Field) -> bool {
    let attr = util::get_field_attr(&f.attrs, "constructor");
    if let Some(Attribute{ meta: Meta::NameValue(MetaNameValue{ ref value, .. }), .. }) = attr.last() {
        if let Expr::Lit(ExprLit{lit: Lit::Bool(LitBool{value, ..}), ..}) = value {
            return *value;
        }
    }
    true
}

pub (crate) fn constructor_derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let vis = &ast.vis;
    let name = &ast.ident;

    let fields = match util::get_named_struct(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error().into(),
    };

    let args = fields.iter().filter_map(|f| {
        if !must_generate(&f) { return None; }
        let ident = util::get_field_ident(f);
        let ty = &f.ty;
        Some(
        quote! {
            #ident : impl std::convert::Into<#ty>
        })
    });
    let names = fields.iter().map(|f| {
        let ident = util::get_field_ident(f);
        if must_generate(&f) {
            quote!( #ident : #ident.into() )
        } else {
            quote!( #ident : std::default::Default::default() )
        }
    });

    let generics = &ast.generics;
    let wher = &generics.where_clause;
    let stripped_generics = get_stripped_generics(generics, false);

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
