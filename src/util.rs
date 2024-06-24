use proc_macro::TokenStream;
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, AngleBracketedGenericArguments, Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Ident, Meta, MetaList, MetaNameValue, Path, PathArguments, Type, TypePath};
use quote::quote;

pub (crate) fn get_inner_ty<'a>(ty: &'a syn::Type, from: &str) -> Option<&'a syn::Type>{
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


pub (crate) fn get_named_struct(input: &DeriveInput) -> syn::Result<&Punctuated<Field,Comma>> {
    let Data::Struct(DataStruct {ref fields,.. }) = input.data else {
        return Err(syn::Error::new(input.span(),
        "The derive macro must be called on a struct"))
    };

    match fields {
        Fields::Named(FieldsNamed { named: fields, .. }) => Ok(fields),
        _ =>  Err(syn::Error::new(fields.span(), "The struct contains unnamed structs"))
    }
}

pub (crate) fn get_field_attr<'a>(field: &'a Field, name: &str) -> Option<&'a Attribute> {
    for attr in &field.attrs {
        let segments = match &attr.meta {
            Meta::Path(Path { segments, .. }) => segments,
            Meta::List(MetaList { path: Path { segments,.. },.. } ) => segments,
            Meta::NameValue(MetaNameValue { path: Path { segments,.. },.. }) => segments,
        };
        if let Some(segment) = segments.last() {
            if segment.ident == Ident::new(name, segment.ident.span()) {
                return Some(attr);
            }
        }
    }
    None
}

#[inline(always)]
pub (crate) fn get_field_ident(field: &Field) -> &syn::Ident {
    let ident = field.ident.as_ref();
    debug_assert!(ident.is_some());
    /* SAFETY: The field is checked to be named. */
    unsafe { ident.unwrap_unchecked() }
}

#[cfg(any(feature = "setters",feature = "getters"))]
pub (crate) fn gen_for_each_field<F>(input: TokenStream, attr: &str, prefix: &str, fun: F) -> proc_macro::TokenStream
where
    F: Fn(&Field) -> proc_macro2::TokenStream
{
    use syn::{Expr, ExprLit, Lit, LitBool};

    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let vis = &ast.vis;
    let fields = match get_named_struct(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error().into()
    };

    let methods = fields.iter().filter_map(|f| {
        if let Some(Attribute{ meta: Meta::NameValue(MetaNameValue{value,..}),..},..) = get_field_attr(f, attr) {
            if let Expr::Lit(ExprLit{lit: Lit::Bool(LitBool{value,..}),..},..) = value {
                if !value { return None; }
            }
        }
        let ident = get_field_ident(f);
        let new_ident = Ident::new(&format!("{prefix}_{}", ident), f.span());
        let code = fun(f);
        Some(quote! {
            #vis fn #new_ident #code
        })
    });

    quote! {
        impl #name {
            #( #methods )*
        }
    }.into()
}
