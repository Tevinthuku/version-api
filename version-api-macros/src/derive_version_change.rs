use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, ExprLit, Lit, LitStr, Meta, parse_macro_input};

pub fn version_change_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match version_change_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn version_change_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ty = &input.ident;
    let description = parse_description_attr(&input.attrs)?;

    Ok(quote! {
        impl version_core::version::VersionChange for #ty {

            fn description() -> &'static str {
                #description
            }
        }
    })
}

fn parse_description_attr(attrs: &[Attribute]) -> syn::Result<LitStr> {
    let description_attr = attrs
        .iter()
        .find(|a| a.path().is_ident("description"))
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing #[description = \"...\"]",
            )
        })?;

    match &description_attr.meta {
        Meta::NameValue(name_value) => match &name_value.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(value),
                ..
            }) => Ok(value.clone()),
            _ => Err(syn::Error::new(
                name_value.value.span(),
                "description value must be a string literal",
            )),
        },
        _ => Err(syn::Error::new(
            description_attr.span(),
            "expected #[description = \"...\"]",
        )),
    }
}
