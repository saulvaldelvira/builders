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
#[proc_macro_derive(Getters, attributes(getter))]
pub fn getters_derive(input: TokenStream) -> TokenStream {
    util::gen_for_each_field(input, "getter", "get", |field| {
        let ident = util::get_field_ident(field);
        let ty = &field.ty;
        quote::quote! {
            (&self) -> & #ty {
                & self.#ident
            }
        }
    })
}

#[cfg(feature = "setters")]
#[proc_macro_derive(Setters, attributes(setter))]
pub fn setters_derive(input: TokenStream) -> TokenStream {
    util::gen_for_each_field(input, "setter", "set", |field| {
        let ident = util::get_field_ident(field);
        let ty = &field.ty;
        quote::quote! {
            (&mut self, #ident: #ty) {
                self.#ident = #ident;
            }
        }
    })
}
