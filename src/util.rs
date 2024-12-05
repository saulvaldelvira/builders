use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, AngleBracketedGenericArguments, Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, GenericParam, Generics, Ident, Meta, MetaList, MetaNameValue, Path, PathArguments, Type, TypePath};
use quote::quote;

pub (crate) fn get_inner_ty<'a>(ty: &'a syn::Type) -> Option<Vec<&'a syn::Type>> {
    let Type::Path(
            TypePath {
                path : Path { ref segments, .. }, ..
        }) = ty else { return None };
    let segment = segments.last()?;

    if let PathArguments::AngleBracketed(
            AngleBracketedGenericArguments { ref args, .. }
            , .. ) = segment.arguments
    {
        return Some(args.into_iter().filter_map(|t| {
            if let GenericArgument::Type(ref t) = t {
                return Some(t);
            }
            None
        }).collect());
    }
    None
}

/* pub (crate) fn get_inner_tys<'a>(ty: &'a syn::Type, from: &'a [&str]) -> Option<(&'a str,Vec<&'a syn::Type>)> { */
/*     from.into_iter().filter_map(|from| { */
/*         get_inner_ty(ty, from).map(|ty| (*from,ty)) */
/*     }).next() */
/* } */

pub (crate) fn get_stripped_generics(generics: &Generics) -> proc_macro2::TokenStream {
    let generics = generics.params.iter().map(|param| {
        let mut param = param.clone();
        match param {
            GenericParam::Lifetime(ref mut l) => l.bounds = Punctuated::new(),
            GenericParam::Type(ref mut t) => t.bounds = Punctuated::new(),
            _ => {}
        };
        quote!(#param)
    });
    quote!(< #( #generics ,)* >)
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

pub (crate) fn get_field_attr<'a>(attrs: &'a [Attribute], name: &str) -> Vec<&'a Attribute> {
    attrs.iter().filter_map(|attr| {
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
        None
    }).collect()
}

#[inline(always)]
pub (crate) fn get_field_ident(field: &Field) -> &syn::Ident {
    let ident = field.ident.as_ref();
    debug_assert!(ident.is_some());
    /* SAFETY: The field is checked to be named. */
    unsafe { ident.unwrap_unchecked() }
}

#[cfg(any(feature = "setters",feature = "getters"))]
pub (crate) fn gen_for_each_field<F>(ast: &DeriveInput, attr: &str, prefix: &str, fun: F) -> proc_macro::TokenStream
where
    F: Fn(&Field,bool) -> proc_macro2::TokenStream
{
    use syn::{Expr, ExprLit, Lit, LitBool};

    let name = &ast.ident;
    let vis = &ast.vis;
    let fields = match get_named_struct(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error().into()
    };

    let methods = fields.iter().filter_map(|f| {
        let mut each = false;
        for attr in get_field_attr(&f.attrs, attr) {
            if let Attribute{ meta: Meta::NameValue(MetaNameValue{value: Expr::Lit(ExprLit { lit, .. }),..}),..} = attr {
                if let Lit::Bool(LitBool{value,..}) = lit {
                    if !value { return None; }
                }
            if let Lit::Str(value) = lit {
                if value.value() == "inner" {
                    each = true;
                }
            }
        }
        }
        let ident = get_field_ident(f);
        let new_ident = Ident::new(&format!("{prefix}{}", ident), f.span());
        let code = fun(f,each);
        Some(quote! {
            #vis fn #new_ident #code
        })
    });

    let generics = &ast.generics;
    let wher = &generics.where_clause;
    let stripped_generics = get_stripped_generics(generics);

    quote! {
        impl #generics #name #stripped_generics #wher {
            #( #methods )*
        }
    }.into()
}

#[cfg(any(feature = "setters",feature = "getters"))]
pub (crate) fn get_prefix<'a>(attrs: &'a [Attribute], def: &'a str, attr_name: &str) -> std::borrow::Cow<'a,str> {
    use proc_macro2::TokenTree;
    use syn::Lit;
    for attr in get_field_attr(attrs, attr_name) {
        if let Attribute { meta: Meta::List(MetaList { ref tokens, .. }), .. } = attr {
            let mut tokens = tokens.clone().into_iter();
            let Some(TokenTree::Ident( ref i )) = tokens.next() else { continue };
            if i == "prefix" {
                tokens.next();
                let Some(TokenTree::Literal(l)) = tokens.next() else { continue };
                let Lit::Str(str) = Lit::new(l) else { continue };
                return str.value().into();
            }
        }
    }
    def.into()
}

