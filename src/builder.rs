use std::borrow::Cow;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Expr, Fields, FieldsNamed, Ident, Lit, Type};

use crate::util::{get_inner_ty, get_stripped_generics};

fn try_find_attr(f: &[syn::Attribute], name: &str) -> Option<()> {
    for attr in f {
        let Ok(list) = attr.meta.require_list() else { continue };
        let segments = &list.path.segments;
        if segments.len() != 1 || segments[0].ident != "builder" { continue }
        let mut tokens = list.tokens.clone().into_iter();
        match tokens.next().unwrap() {
            TokenTree::Ident(ref i) => if i == name {
                return Some(())
            },
            tt => panic!("Expected \"{name}\", found: {tt}"),
        }
    }
    None
}

fn find_attr_nameval(f: &[syn::Attribute], name: &str) -> Result<TokenStream,proc_macro2::TokenStream> {
    for attr in f {
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

fn find_attr_nameval_lit(f: &[syn::Attribute], name: &str) -> Result<Lit,proc_macro2::TokenStream> {
    let token = find_attr_nameval(f, name)?;
    let lit: Lit = parse_quote!(#token);
    Ok(lit)
}

fn find_attr_nameval_expr(f: &[syn::Attribute], name: &str) -> Result<Expr,proc_macro2::TokenStream> {
    let token = find_attr_nameval(f, name)?;
    let lit: Expr = parse_quote!(#token);
    Ok(lit)
}

fn find_attr_nameval_bool(f: &[syn::Attribute], name: &str) -> Result<bool,proc_macro2::TokenStream> {
    if try_find_attr(f, name).is_some() { return Ok(true) }
    let token = find_attr_nameval(f, name)?;
    let lit: Lit = parse_quote!(#token);
    let Lit::Bool(b) = lit else { return Err(quote! { compile_error!("Expected boolean attribute"); })};
    Ok(b.value)
}

fn is_optional(f: &syn::Field) -> bool {
    find_attr_nameval_bool(&f.attrs, "optional").unwrap_or_default()
}

fn is_disabled(f: &syn::Field) -> bool {
    find_attr_nameval_bool(&f.attrs, "disabled").unwrap_or_default()
}

fn is_each(f: &syn::Field) -> bool {
    matches!(find_attr_nameval_lit(&f.attrs, "vec"), Ok(Lit::Str(_)))
    || matches!(find_attr_nameval_lit(&f.attrs, "map"), Ok(Lit::Str(_)))
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
    let builder_fields = fields.iter().filter(|f| !is_disabled(f)).map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if is_optional(f) || is_each(f) {
            quote!( #name: #ty )
        } else {
            quote!( #name: ::core::option::Option<#ty> )
        }
    });
    let builder_methods = fields.iter().filter(|f| !is_disabled(f)).map(|f| {
        let field_name = &f.ident;
        match find_attr_nameval_lit(&f.attrs, "vec") {
            Ok(Lit::Str(lit)) => {
                let arg = Ident::new(&lit.value(), lit.span());
                let set_arg = Ident::new(&format!("set_{}", &lit.value()), lit.span());
                let tys = get_inner_ty(&f.ty).expect("Expected at least one generic argument");
                let inner = tys.first();
                quote! {
                    pub fn #arg(mut self, #arg: impl ::core::convert::Into<#inner>) -> Self {
                        self.#field_name.push(#arg.into());
                        self
                    }

                    pub fn #set_arg(&mut self, #arg: impl ::core::convert::Into<#inner>) -> &mut Self {
                        self.#field_name.push(#arg.into());
                        self
                    }
                }
            },
            Ok(_) => {
                quote! {
                    compile_error!("Expected \"each\" to be a string");
                }
            },
            Err(_) => {
                match find_attr_nameval_lit(&f.attrs, "map") {
                    Ok(Lit::Str(lit)) => {
                        let arg = Ident::new(&lit.value(), lit.span());
                        let set_arg = Ident::new(&format!("set_{}", &lit.value()), lit.span());
                        let [first, second] = get_inner_ty(&f.ty).expect("Expected at least one generic argument")[..2] else { panic!() };
                        quote! {
                            pub fn #arg(mut self, #arg: impl ::core::convert::Into<#first>, val: impl ::core::convert::Into<#second>) -> Self {
                                self.#field_name.insert(#arg.into(),val.into());
                                self
                            }

                            pub fn #set_arg(&mut self, #arg: impl ::core::convert::Into<#first>, val: impl ::core::convert::Into<#second>) -> &mut Self {
                                self.#field_name.insert(#arg.into(),val.into());
                                self
                            }
                        }
                    },
                    Ok(_) => {
                        quote! {
                            compile_error!("Expected \"each\" to be a string");
                        }
                    },
                    Err(err) => {
                        let ty: Cow<Type> = if is_optional(f) {
                            let t = get_inner_ty(&f.ty).map(|l| l[0]).unwrap_or(&f.ty);
                            let t: Type = parse_quote!(#t);
                            Cow::Owned(t)
                        } else {
                            Cow::Borrowed(&f.ty)
                        };
                        let fname = field_name.as_ref().unwrap();
                        let set_name = Ident::new(&format!("set_{}", &fname.to_string()), fname.span());
                        quote! {
                            #err
                            pub fn #field_name(mut self, #field_name: impl ::core::convert::Into<#ty>) -> Self {
                                self.#field_name = ::core::option::Option::Some(#field_name.into());
                                self
                            }

                            pub fn #set_name(&mut self, #field_name: impl ::core::convert::Into<#ty>) -> &mut Self {
                                self.#field_name = ::core::option::Option::Some(#field_name.into());
                                self
                            }
                        }
                    }
                }
            }
        }
    });
    let empty_fields = fields.iter().filter(|f| !is_disabled(f)).map(|f| {
        let field_name = &f.ident;
        if is_each(f) {
            return quote! { #field_name : ::core::default::Default::default() };
        }
        quote! { #field_name : ::core::option::Option::None }
    });
    let generics = &ast.generics;
    let wher = &generics.where_clause;
    let stripped_generics = get_stripped_generics(generics, false);
    let generics_no_defaults = get_stripped_generics(generics, true);

    let vis = &ast.vis;

    let build_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! { #field_name }
    });


    let is_infallible = find_attr_nameval_bool(&ast.attrs, "infallible").is_ok_and(|v| v);
    if is_infallible {
        for field in fields.iter() {
            if !is_optional(field) && !is_each(field) && find_attr_nameval_expr(&field.attrs, "def").is_err() {
                return quote! {compile_error!("Infallible builder must have all it's fields \
                    marked as either optional, or provide with a default value for them"); }.into()
            }
        }
    }

    let build_fields_let = fields.iter().map(|f| {
        let field_name = &f.ident;
        let expr =
            if is_disabled(f) {
                let Ok(lit) = find_attr_nameval_expr(&f.attrs, "def") else {
                    panic!("Disabled field must have a builder(def = ...) attribute");
                };
                quote! { #lit }
            }
            else if is_optional(f) {
                quote! { self.#field_name }
            } else if let Ok(lit) = find_attr_nameval_expr(&f.attrs, "def"){
                quote! { self.#field_name.unwrap_or_else(|| #lit .into()) }
            }  else if is_each(f) {
                quote! { self. #field_name }
            } else {
                quote! { self.#field_name.ok_or(stringify!("{} is not set", #field_name))? }
            };
        quote! { let #field_name = #expr ; }
    });

    let generics_params = &ast.generics.params;

    let clone_where_token = if wher.is_none() { quote! { where }} else { quote! {} };

    let must_clone = find_attr_nameval_bool(&ast.attrs, "clone").is_ok_and(|v| v);

    let mut clone_impl = quote!{};

    if must_clone {
        let where_clone = ast.generics.type_params().map(|tp| {
            let id = &tp.ident;
            quote! { #id : Clone }
        });
        let fields_clone = fields.iter().filter(|f| !is_disabled(f)).map(|field| {
            let name = &field.ident;
            quote! { #name : self . #name .clone() }
        });

        clone_impl = quote! {
            impl<#generics_params> Clone for #builder_name #stripped_generics
                #wher
                #clone_where_token
                #( #where_clone, )*
            {
                fn clone(&self) -> Self {
                    Self {
                        #(#fields_clone ,)*
                    }
                }
            }
        };
    }

    let build_fn_return = if is_infallible {
        quote! { #ident #stripped_generics }
    } else {
        quote! { ::core::result::Result<#ident #stripped_generics, &'static str>  }
    };

    let mut build_self_struct = quote! {
        #ident {
            #( #build_fields ,)*
        }
    };

    if !is_infallible {
        build_self_struct = quote! { Ok( #build_self_struct ) };
    }

    quote! {
        #vis struct #builder_name #generics {
            #( #builder_fields ,)*
        }

        #clone_impl

        impl #generics_no_defaults #builder_name #stripped_generics #wher {
            #vis fn build(self) -> #build_fn_return {
                #( #build_fields_let )*
                #build_self_struct
            }

            #( #builder_methods )*
        }

        impl #generics_no_defaults #ident #stripped_generics #wher {
            #vis fn builder() -> #builder_name #stripped_generics {
                #builder_name {
                    #( #empty_fields ,)*
                }
            }
        }
    }.into()
}
