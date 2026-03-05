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
                "ApiVersionId can only be derived on enums",
            ));
        }
    };

    let mut variant_idents = Vec::with_capacity(data.variants.len());
    let mut version_strings = Vec::with_capacity(data.variants.len());
    let mut versions = Vec::with_capacity(data.variants.len());

    struct VersionInfo {
        raw_value: LitStr,
        version_id: version_id::VersionId,
    }
    for variant in &data.variants {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new(
                variant.ident.span(),
                "ApiVersionId variants must be unit variants (no fields)",
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

        version_strings.push(version_lit.clone());

        let version_id = version_id::VersionId::try_from(version_lit.value()).map_err(|e| {
            syn::Error::new(variant.ident.span(), format!("invalid version: {}", e))
        })?;

        variant_idents.push(&variant.ident);
        versions.push(VersionInfo {
            raw_value: version_lit,
            version_id,
        });
    }

    for window in versions.windows(2) {
        let from_version = &window[0];
        let to_version = &window[1];
        if from_version.version_id <= to_version.version_id {
            return Err(syn::Error::new(
                to_version.raw_value.span(),
                format!(
                    "versions must be listed newest-first; \"{}\" is not older than \"{}\"",
                    to_version.raw_value.value(),
                    from_version.raw_value.value(),
                ),
            ));
        }
    }

    Ok(quote! {
        impl #enum_name {
            pub const ALL: &[#enum_name] = &[
                #(#enum_name::#variant_idents),*
            ];

            pub fn as_str(&self) -> &str {
                match self {
                    #(#enum_name::#variant_idents => #version_strings),*
                }
            }

            fn as_version_id(&self) -> version_id::VersionId {
                match self {
                    #(#enum_name::#variant_idents => {
                        version_id::VersionId::try_from(#version_strings)
                            .expect("already validated at compile time")
                    }),*
                }
            }
        }


        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::convert::From<#enum_name> for version_id::VersionId {
            fn from(v: #enum_name) -> Self {
                v.as_version_id().clone()
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
