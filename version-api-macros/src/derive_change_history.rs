use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Attribute;
use syn::DeriveInput;
use syn::Expr;
use syn::Token;
use syn::Type;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;

use crate::TransformDirection;

pub fn change_history_derive_impl(
    input: TokenStream,
    resource_type: TransformDirection,
) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match change_history_impl(&input, resource_type) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn change_history_impl(
    input: &DeriveInput,
    resource_type: TransformDirection,
) -> syn::Result<proc_macro2::TokenStream> {
    let change_history_type = &input.ident;
    let head = parse_head_attr(&input.attrs)?;
    let changes = parse_changes_attr(&input.attrs)?;
    if changes.is_empty() {
        return Err(syn::Error::new(
            change_history_type.span(),
            "#[changes(...)] list must not be empty",
        ));
    }

    let changes_types = changes.iter().map(|data| data.the_type.clone()).collect::<Vec<_>>();

    let mut chain: Vec<Type> = Vec::with_capacity(changes.len() + 1);
    chain.push(head.clone());
    chain.extend(changes_types);

    let mut from_assertions = Vec::new();

    let mut transformer_structs = Vec::new();
    let mut transformer_impls = Vec::new();
    let mut register_entries = Vec::new();

    // For responses, the chain is Head → Oldest (downgrade): we convert from the
    // latest type toward older versions, so the natural order [Head, ...changes] is
    // already correct.
    //
    // For requests, the direction is reversed — we upgrade from an older request
    // body toward the Head type that the handler expects. The `#[changes]` list
    // is still authored newest-first (matching the response convention), so we
    // reverse the chain here to get Oldest → Head, making each `From` step go
    // from an older type to the next newer one.
    let transform_steps = match resource_type {
        TransformDirection::Request => chain
            .iter()
            .rev()
            .tuple_windows::<(_, _)>()
            .zip(changes.iter().rev().map(|data| data.version.clone()))
            .collect::<Vec<_>>(),
        TransformDirection::Response => chain
            .iter()
            .tuple_windows::<(_, _)>()
            .zip(changes.iter().map(|data| data.version.clone()))
            .collect::<Vec<_>>(),
    };

    for ((from_type, to_type), version) in transform_steps.iter() {
        from_assertions.push(quote! {
            _assert_from::<#from_type, #to_type>();
        });
        let mut from_type_str = from_type.into_token_stream().to_string();
        // Sanitize the type name for use as a Rust identifier (e.g. `Foo<Bar>` → `FooBar`)
        from_type_str.retain(|c| c.is_ascii_alphabetic() || c == '_');

        let mut to_type_str = to_type.into_token_stream().to_string();
        // Sanitize the type name for use as a Rust identifier (e.g. `Foo<Bar>` → `FooBar`)
        to_type_str.retain(|c| c.is_ascii_alphabetic() || c == '_');

        let transformer_name =
            format_ident!("__From_{}_To_{}_Transformer", from_type_str, to_type_str,);

        transformer_structs.push(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #transformer_name;
        });

        let resource_type = match resource_type {
            TransformDirection::Request => {
                quote! { version_core::version::ResourceType::Request }
            }
            TransformDirection::Response => {
                quote! { version_core::version::ResourceType::Response }
            }
        };

        transformer_impls.push(quote! {
            impl version_core::version::VersionChangeTransformer for #transformer_name {
                type Input = #from_type;
                type Output = #to_type;

                fn resource_type(&self) -> version_core::version::ResourceType {
                    #resource_type
                }

                fn description(&self) -> &str {
                    <#to_type as version_core::version::VersionChange>::description()
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
                id: ::std::convert::Into::<version_id::VersionId>::into(#version),
                changes: vec![::std::boxed::Box::new(#transformer_name)],
            });
        });
    }

    let version_ids = changes.into_iter().map(|data| data.version).map(|version| {
        quote! { ::std::convert::Into::<version_id::VersionId>::into(#version) }
    });

    Ok(quote! {
        #(#transformer_structs)*
        #(#transformer_impls)*

        impl version_core::version::RequestChangeHistory for #change_history_type {
            type Head = #head;

            fn version_ids() -> ::std::vec::Vec<version_id::VersionId> {
                ::std::vec![#(#version_ids),*]
            }

            fn register(registry: &mut version_core::registry::ResourceRegistry) -> Result<(), Box<dyn ::std::error::Error>> {
                let version_ids = Self::version_ids();

                for window in version_ids.windows(2) {
                    if window[0] <= window[1] {
                        return Err(Box::new(
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::InvalidInput,
                                format!("changes must be ordered newest-first by `below` version; got {:?} then {:?}", window[0], window[1])
                            )
                        ));
                    }
                }
                #(#register_entries)*

                Ok(())
            }
        }

        impl #change_history_type {
            pub fn version_ids() -> ::std::vec::Vec<version_id::VersionId> {
                <Self as version_core::version::RequestChangeHistory>::version_ids()
            }

            pub fn register(registry: &mut version_core::registry::ResourceRegistry) -> Result<(), Box<dyn ::std::error::Error>> {
                <Self as version_core::version::RequestChangeHistory>::register(registry)
            }
        }

        const _: () = {
            fn _assert_from<T, U: ::std::convert::From<T>>() {}
            fn _check_change_chain() {
                #(#from_assertions)*
            }
        };
    })
}

fn parse_head_attr(attrs: &[Attribute]) -> syn::Result<Type> {
    let head_attr = attrs
        .iter()
        .find(|a| a.path().is_ident("head"))
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "missing #[head(...)]"))?;
    head_attr.parse_args::<Type>()
}

struct ChangesArg {
    version: Expr,  // eg: MyApiVersions::V1_0_0
    the_type: Type, // eg: CollapseUserAddressesToStrings
}

impl Parse for ChangesArg {
    // we are parsing the following structure: below(MyApiVersions::V1_0_0) => CollapseUserAddressesToStrings
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _below_keyword: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let version: Expr = content.parse()?;
        input.parse::<Token![=>]>()?;
        let the_type: Type = input.parse()?;
        Ok(ChangesArg { version, the_type })
    }
}

fn parse_changes_attr(attrs: &[Attribute]) -> syn::Result<Vec<ChangesArg>> {
    let changes_attr = attrs.iter().find(|a| a.path().is_ident("changes")).ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "missing #[changes(...)]")
    })?;
    let args: Punctuated<ChangesArg, Token![,]> =
        changes_attr.parse_args_with(Punctuated::<ChangesArg, Token![,]>::parse_terminated)?;
    Ok(args.into_iter().collect())
}
