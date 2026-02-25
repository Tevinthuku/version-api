use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, LitStr, Token, parse_macro_input};

struct VersionEntry {
    version: LitStr,
    ty: Ident,
}

impl Parse for VersionEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let version: LitStr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let ty: Ident = input.parse()?;
        Ok(VersionEntry { version, ty })
    }
}

struct ResponseArgs {
    versions: Vec<VersionEntry>,
}

impl Parse for ResponseArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "changed_in" {
            return Err(syn::Error::new(ident.span(), "expected `changed_in`"));
        }

        let content;
        syn::parenthesized!(content in input);
        let versions: Punctuated<VersionEntry, Token![,]> =
            content.parse_terminated(VersionEntry::parse, Token![,])?;

        if versions.is_empty() {
            return Err(syn::Error::new(
                ident.span(),
                "changed_in list must not be empty",
            ));
        }

        Ok(ResponseArgs {
            versions: versions.into_iter().collect(),
        })
    }
}

/// Derive macro for the head response struct.
///
/// Usage:
/// ```ignore
/// #[derive(VersionedResponse)]
/// #[response(changed_in(
///     "2025-02-01" => V2,
///     "2025-01-10" => V1,
/// ))]
/// struct Head { ... }
/// ```
///
/// The versions list is ordered newest-to-oldest. Each adjacent pair
/// (Head, V2), (V2, V1) requires a `From` impl.
#[proc_macro_derive(VersionedResponse, attributes(response))]
pub fn versioned_response_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match versioned_response_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn versioned_response_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let head = &input.ident;

    let response_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("response"))
        .ok_or_else(|| {
            syn::Error::new_spanned(head, "#[response(changed_in(...))] attribute is required")
        })?;

    let args: ResponseArgs = response_attr.parse_args()?;

    for window in args.versions.windows(2) {
        if window[0].version.value() <= window[1].version.value() {
            return Err(syn::Error::new(
                window[1].version.span(),
                format!(
                    "version \"{}\" must be strictly older than \"{}\"; versions must be listed newest-first",
                    window[1].version.value(),
                    window[0].version.value(),
                ),
            ));
        }
    }

    // Build the chain: [Head, V_newest, ..., V_oldest]
    let mut chain: Vec<&Ident> = Vec::with_capacity(args.versions.len() + 1);
    chain.push(head);
    chain.extend(args.versions.iter().map(|e| &e.ty));

    let mut transformer_structs = Vec::new();
    let mut transformer_impls = Vec::new();
    let mut register_entries = Vec::new();

    for (i, window) in chain.windows(2).enumerate() {
        let from_type = window[0];
        let to_type = window[1];
        let version_lit = &args.versions[i].version;

        let transformer_name = format_ident!("__{}To{}Transformer", from_type, to_type);

        transformer_structs.push(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #transformer_name;
        });

        transformer_impls.push(quote! {
            impl version_core::version::VersionChangeSetTransformer for #transformer_name {
                type Input = #from_type;
                type Output = #to_type;

                fn description(&self) -> &str {
                    concat!("Transform ", stringify!(#from_type), " to ", stringify!(#to_type))
                }

                fn head_version(&self) -> ::std::any::TypeId {
                    ::std::any::TypeId::of::<#head>()
                }

                fn transform(
                    &self,
                    input: #from_type,
                ) -> ::std::result::Result<#to_type, Box<dyn ::std::error::Error>> {
                    ::std::result::Result::Ok(
                        <#to_type as ::std::convert::From<#from_type>>::from(input)
                    )
                }
            }
        });

        register_entries.push(quote! {
            registry.register(version_core::version::Version {
                id: version_core::version::VersionId::from(#version_lit),
                changes: vec![::std::boxed::Box::new(#transformer_name)],
            });
        });
    }

    let version_lits: Vec<&LitStr> = args.versions.iter().map(|e| &e.version).collect();

    let mut from_assertions = Vec::new();
    for window in chain.windows(2) {
        let from_type = window[0];
        let to_type = window[1];
        from_assertions.push(quote! {
            _assert_from::<#from_type, #to_type>();
        });
    }

    Ok(quote! {
        #(#transformer_structs)*

        #(#transformer_impls)*

        impl #head {
            pub fn version_ids() -> ::std::vec::Vec<version_core::version::VersionId> {
                ::std::vec![
                    #(version_core::version::VersionId::from(#version_lits)),*
                ]
            }

            pub fn register_versions(
                registry: &mut version_core::registry::ApiResponseResourceRegistry,
            ) {
                #(#register_entries)*
            }
        }

        const _: () = {
            fn _assert_from<T, U: ::std::convert::From<T>>() {}
            fn _check_version_chain() {
                #(#from_assertions)*
            }
        };
    })
}
