use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident, LitStr};

use crate::util;

fn find_attr_each(f: &syn::Field) -> Result<LitStr,proc_macro2::TokenStream> {
    for attr in &f.attrs {
        let list = attr.meta.require_list().map_err(|err| err.to_compile_error())?;
        let segments = &list.path.segments;
        if segments.len() != 1 || segments[0].ident != "builder" { return Err(TokenStream::new()); }
        let mut tokens = list.tokens.clone().into_iter();
        match tokens.next().unwrap() {
            TokenTree::Ident(ref i) => if i != "each" {
                return Err(syn::Error::new(
                            i.span(),
                            format!("expected `builder(each = \"...\")`")).into_compile_error());
            },
            tt => panic!("Expected \"each\", found: {tt}"),
        }
        match tokens.next().unwrap() {
            TokenTree::Punct(ref p) => assert_eq!(p.as_char(), '='),
            tt => panic!("Expected '=', found: {tt}"),
        }
        let lit = match tokens.next().unwrap() {
            TokenTree::Literal(lit) => lit,
            _ => panic!("Expected string"),
        };
        match syn::Lit::new(lit) {
            syn::Lit::Str(s) => return Ok(s),
            _ => panic!("Expected string"),
        };
    }
    Err(TokenStream::new())
}

pub (crate) fn builder_derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let builder_name = format!("{ident}Builder");
    let builder_name = Ident::new(&builder_name, ident.span());

    let fields=
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data {
        named
    } else {
        unimplemented!()
    };
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if util::get_inner_ty(ty,"Option").is_some() ||
            find_attr_each(f).is_ok() {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });
    let builder_methods = fields.iter().map(|f| {
        let field_name = &f.ident;
        match find_attr_each(f) {
            Ok(lit) => {
                let arg = Ident::new(&lit.value(), lit.span());
                let inner_ty = util::get_inner_ty(&f.ty, "Vec");
                return quote! {
                    pub fn #arg(&mut self, #arg: impl std::convert::Into<#inner_ty>) -> &mut Self {
                        self.#field_name.push(#arg.into());
                        self
                    }
                }
            },
            Err(err) => {
                let ty = util::get_inner_ty(&f.ty,"Option").unwrap_or(&f.ty);
                quote! {
                    #err
                    pub fn #field_name(&mut self, #field_name: impl std::convert::Into<#ty>) -> &mut Self {
                        self.#field_name = std::option::Option::Some(#field_name.into());
                        self
                    }
                }
            }
        }
    });
    let empty_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        if find_attr_each(f).is_ok() {
            quote! { #field_name : Vec::new() }
        } else {
            quote! { #field_name : std::option::Option::None }
        }
    });
    let build_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let expr =
        if util::get_inner_ty(&f.ty,"Option").is_some()
            || find_attr_each(f).is_ok()
        {
            quote! { self.#field_name.clone() }
        } else {
            quote! { self.#field_name.clone().ok_or(stringify!("{} is not set", #field_name))? }
        };
        quote! { #field_name: #expr }
    });
    quote! {
        pub struct #builder_name {
            #( #builder_fields ,)*
        }

        impl #builder_name {
            pub fn build(&self) -> std::result::Result<#ident,std::boxed::Box<dyn std::error::Error>> {
                Ok(#ident {
                    #( #build_fields ,)*
                })
            }
            #( #builder_methods )*
        }

        impl #ident {
            fn builder() -> #builder_name {
                #builder_name {
                    #( #empty_fields ,)*
                }
            }
        }
    }.into()
}
