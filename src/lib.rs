mod util;
#[cfg(feature = "builder")]
mod builder;

use proc_macro::TokenStream;

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

    util::gen_for_each_field(&ast, "getter", &prefix, |field,each| {
        let ident = util::get_field_ident(field);
        let ty = &field.ty;
        if each {
            if let Some((out,inner)) = util::get_inner_tys(ty,&["Vec","HashMap"]) {
                let first = inner[0];
                return match out {
                    "Vec" => {
                        quote! {
                            (&self, i: usize) -> std::option::Option<&#first> {
                                self.#ident.get(i)
                            }
                        }
                    },
                    "HashMap" => {
                        let second = inner[1];
                        quote! {
                            (&self, key: & #first) -> std::option::Option<&#second> {
                                self.#ident.get(key)
                            }
                        }
                    },
                    _ => unreachable!()
                }
            }
        }
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
            (&mut self, #ident: impl std::convert::Into<#ty>) {
                self.#ident = #ident.into();
            }
        }
    })
}

#[cfg(feature = "constructor")]
mod constructor;

#[cfg(feature = "constructor")]
#[proc_macro_derive(Constructor)]
pub fn constructor_derive(input: TokenStream) -> TokenStream {
    constructor::constructor_derive_impl(input)
}

