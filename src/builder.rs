use std::borrow::Cow;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Expr, Fields, FieldsNamed, Ident, Lit, LitBool, Type};

use crate::util::{get_inner_ty, get_inner_tys, get_stripped_generics};

fn find_attr_nameval(f: &syn::Field, name: &str) -> Result<TokenStream,proc_macro2::TokenStream> {
    for attr in &f.attrs {
        let Ok(list) = attr.meta.require_list() else { continue };
        let segments = &list.path.segments;
        if segments.len() != 1 || segments[0].ident != "builder" { return Err(TokenStream::new()); }
        let mut tokens = list.tokens.clone().into_iter();
        match tokens.next().unwrap() {
            TokenTree::Ident(ref i) => if i != name {
                continue
            },
            tt => panic!("Expected \"{name}\", found: {tt}"),
        }
        match tokens.next().unwrap() {
            TokenTree::Punct(ref p) => assert_eq!(p.as_char(), '='),
            tt => panic!("Expected '=', found: {tt}"),
        }
        return Ok(tokens.next().unwrap().into());
    }
    Err(TokenStream::new())
}

fn find_attr_nameval_lit(f: &syn::Field, name: &str) -> Result<Lit,proc_macro2::TokenStream> {
    let token = find_attr_nameval(f, name)?;
    let lit: Lit = parse_quote!(#token);
    Ok(lit)
}

fn find_attr_nameval_expr(f: &syn::Field, name: &str) -> Result<Expr,proc_macro2::TokenStream> {
    let token = find_attr_nameval(f, name)?;
    let lit: Expr = parse_quote!(#token);
    Ok(lit)
}

fn is_optional(f: &syn::Field) -> bool {
    if let Ok(Lit::Bool(LitBool{value,..})) = find_attr_nameval_lit(f, "optional") {
        value
    } else { false }
}

fn is_each(f: &syn::Field) -> bool {
    matches!(find_attr_nameval_lit(f, "each"), Ok(Lit::Str(_)))
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
        if is_optional(f) {
            quote!( #name: #ty )
        } else {
            quote!( #name: std::option::Option<#ty> )
        }
    });
    let builder_methods = fields.iter().map(|f| {
        let field_name = &f.ident;
        match find_attr_nameval_lit(f, "each") {
            Ok(Lit::Str(lit)) => {
                let arg = Ident::new(&lit.value(), lit.span());
                let (ty,inner_ty) = get_inner_tys(&f.ty, &["Vec","HashMap"]).unwrap();
                let first = inner_ty.first();
                match ty {
                    "Vec" => {
                        quote! {
                            pub fn #arg(&mut self, #arg: impl std::convert::Into<#first>) -> &mut Self {
                                self.#field_name.as_mut().unwrap().push(#arg.into());
                                self
                            }
                        }
                    },
                    "HashMap" => {
                        let second = inner_ty[1];
                        quote! {
                            pub fn #arg(&mut self, #arg: impl std::convert::Into<#first>, val: impl std::convert::Into<#second>) -> &mut Self {
                                self.#field_name.as_mut().unwrap().insert(#arg.into(),val.into());
                                self
                            }
                        }
                    },
                   _ => unimplemented!()
                }
            },
            Ok(_) => {
                quote! {
                    compile_error!("Expected \"each\" to be a string");
                }
            },
            Err(err) => {
                let ty: Cow<Type> = if is_optional(f) {
                    let t = get_inner_ty(&f.ty,"Option").map(|l| l[0]).unwrap_or(&f.ty);
                    let t: Type = parse_quote!(#t);
                    Cow::Owned(t)
                } else {
                    Cow::Borrowed(&f.ty)
                };
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
        if is_each(f) {
            let Some((out,_)) = get_inner_tys(&f.ty, &["Vec","HashMap"]) else { unreachable!() };
            let out = match out {
                "Vec" => quote!(Vec),
                "HashMap" => quote!(HashMap),
                _ => unreachable!()
            };
            return quote! { #field_name : std::option::Option::Some(#out ::new()) };
        }
        quote! { #field_name : std::option::Option::None }
    });
    let generics = &ast.generics;
    let wher = &generics.where_clause;
    let stripped_generics = get_stripped_generics(generics);
    let vis = &ast.vis;

    let build_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! { #field_name }
    });
    let build_fields_let = fields.iter().map(|f| {
        let field_name = &f.ident;
        let expr =
            if is_optional(f) {
                quote! { self.#field_name.take() }
            } else if let Ok(lit) = find_attr_nameval_expr(f, "def"){
                quote! { self.#field_name.take().unwrap_or_else(|| #lit .into()) }
            } else {
                quote! { self.#field_name.take().ok_or(stringify!("{} is not set", #field_name))? }
            };
        quote! { let #field_name = #expr ; }
    });

    quote! {
        #[derive(Clone)]
        #vis struct #builder_name #generics {
            #( #builder_fields ,)*
        }

        impl #generics #builder_name #stripped_generics #wher {
            #vis fn build(&mut self) -> std::result::Result<#ident #stripped_generics,std::boxed::Box<dyn std::error::Error>> {
                #( #build_fields_let )*
                Ok(#ident {
                    #( #build_fields ,)*
                })
            }

            #( #builder_methods )*
        }

        impl #generics #ident #stripped_generics #wher {
            #vis fn builder() -> #builder_name #stripped_generics {
                #builder_name {
                    #( #empty_fields ,)*
                }
            }
        }
    }.into()
}
