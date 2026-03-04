use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

pub fn api_version_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match api_version_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn api_version_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;

    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new(
                enum_name.span(),
                "ApiVersion can only be derived on enums",
            ));
        }
    };

    let mut variant_idents = Vec::new();
    let mut version_strings = Vec::new();

    for variant in &data.variants {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new(
                variant.ident.span(),
                "ApiVersion variants must be unit variants (no fields)",
            ));
        }

        let version_lit = variant
            .attrs
            .iter()
            .find(|a| a.path().is_ident("version"))
            .ok_or_else(|| {
                syn::Error::new(
                    variant.ident.span(),
                    "missing #[version(\"...\")] attribute",
                )
            })?
            .parse_args::<LitStr>()?;

        variant_idents.push(&variant.ident);
        version_strings.push(version_lit);
    }

    for window in version_strings.windows(2) {
        if window[0].value() <= window[1].value() {
            return Err(syn::Error::new(
                window[1].span(),
                format!(
                    "versions must be listed newest-first; \"{}\" is not older than \"{}\"",
                    window[1].value(),
                    window[0].value(),
                ),
            ));
        }
    }

    Ok(quote! {
        impl #enum_name {
            pub const ALL: &[#enum_name] = &[
                #(#enum_name::#variant_idents),*
            ];

            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#enum_name::#variant_idents => #version_strings),*
                }
            }
        }


        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::convert::From<#enum_name> for version_core::version::VersionId {
            fn from(v: #enum_name) -> Self {
                version_core::version::VersionId::from(v.as_str())
            }
        }

        impl ::std::str::FromStr for #enum_name {
            type Err = ::std::string::String;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    #(#version_strings => ::std::result::Result::Ok(#enum_name::#variant_idents),)*
                    other => ::std::result::Result::Err(
                        ::std::format!("unknown API version: {}", other)
                    ),
                }
            }
        }

    })
}
