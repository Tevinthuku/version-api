use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Ident, Lit, LitStr, Meta, Token, parse_macro_input,
};

pub fn version_change_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match version_change_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn version_change_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ty = &input.ident;
    let below = parse_below_attr(&input.attrs)?;
    let description = parse_description_attr(&input.attrs)?;

    Ok(quote! {
        impl version_core::version::VersionChange for #ty {
            fn below_version() -> version_id::VersionId {
                ::std::convert::Into::<version_id::VersionId>::into(#below)
            }

            fn description() -> &'static str {
                #description
            }
        }
    })
}

struct BelowArg {
    key: Ident,
    value: Expr,
}

impl Parse for BelowArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(BelowArg { key, value })
    }
}

fn parse_below_attr(attrs: &[Attribute]) -> syn::Result<Expr> {
    let version_attr = attrs
        .iter()
        .find(|a| a.path().is_ident("version"))
        .ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "missing #[version(...)]")
        })?;

    let args: Punctuated<BelowArg, Token![,]> =
        version_attr.parse_args_with(Punctuated::<BelowArg, Token![,]>::parse_terminated)?;
    let below = args.into_iter().find(|a| a.key == "below").ok_or_else(|| {
        syn::Error::new(version_attr.span(), "expected #[version(below = \"...\")]")
    })?;
    Ok(below.value)
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
