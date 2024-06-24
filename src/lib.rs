use core::panic;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Fields, FieldsNamed, GenericArgument, Ident, LitStr, Path, PathArguments, Type, TypePath};

fn get_inner_ty<'a>(ty: &'a syn::Type, from: &str) -> Option<&'a syn::Type>{
    let Type::Path(
            TypePath {
                path : Path { ref segments, .. }, ..
        }) = ty else { return None };
    if segments.len() != 1 || segments[0].ident != from {
        return None;
    }
    if let PathArguments::AngleBracketed(
            AngleBracketedGenericArguments { ref args, .. }
            , .. ) = segments[0].arguments
    {
        if args.len() != 1 {
            return None;
        }
        let inner_ty = args.first().unwrap();
        if let GenericArgument::Type(ref t) = inner_ty {
            return Some(t);
        }
    }
    None
}

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

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{name}Builder");
    let bname = Ident::new(&bname, name.span());

    let fields=
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data {
        named
    } else {
        unimplemented!()
    };
    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if get_inner_ty(ty,"Option").is_some() ||
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
    let methods = fields.iter().map(|f| {
        let field_name = &f.ident;
        match find_attr_each(f) {
            Ok(lit) => {
                let arg = Ident::new(&lit.value(), lit.span());
                let inner_ty = get_inner_ty(&f.ty, "Vec");
                return quote! {
                    pub fn #arg(&mut self, #arg: impl std::convert::Into<#inner_ty>) -> &mut Self {
                        self.#field_name.push(#arg.into());
                        self
                    }
                }
            },
            Err(err) => {
                let name = &f.ident;
                let ty = get_inner_ty(&f.ty,"Option").unwrap_or(&f.ty);
                quote! {
                    #err
                    pub fn #name(&mut self, #name: impl std::convert::Into<#ty>) -> &mut Self {
                        self.#name = std::option::Option::Some(#name.into());
                        self
                    }
                }
            }
        }
    });
    let empty_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if find_attr_each(f).is_ok() {
            quote! { #name : Vec::new() }
        } else {
            quote! { #name : std::option::Option::None }
        }
    });
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let expr =
        if get_inner_ty(&f.ty,"Option").is_some()
            || find_attr_each(f).is_ok()
        {
            quote! { self.#name.clone() }
        } else {
            quote! { self.#name.clone().ok_or(stringify!("{} is not set", #name))? }
        };
        quote! { #name: #expr }
    });
    let builder = quote! {
        pub struct #bname {
            #(#optionized ,)*
            }

        impl #bname {
            pub fn build(&self) -> std::result::Result<#name,std::boxed::Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields ,)*
                })
            }
            #(#methods)*
        }

        impl #name {
            fn builder() -> #bname {
                #bname {
                    #(#empty_fields ,)*
                }
            }
        }
    };

    builder.into()
}
