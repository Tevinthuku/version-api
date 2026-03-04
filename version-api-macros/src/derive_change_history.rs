use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, DeriveInput, Token, Type, parse_macro_input};

pub fn change_history_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match change_history_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn change_history_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let change_history_type = &input.ident;
    let head = parse_head_attr(&input.attrs)?;
    let changes = parse_changes_attr(&input.attrs)?;
    if changes.is_empty() {
        return Err(syn::Error::new(
            change_history_type.span(),
            "#[changes(...)] list must not be empty",
        ));
    }

    let mut chain: Vec<Type> = Vec::with_capacity(changes.len() + 1);
    chain.push(head.clone());
    chain.extend(changes.iter().cloned());

    let mut transformer_structs = Vec::new();
    let mut transformer_impls = Vec::new();
    let mut register_entries = Vec::new();

    for window in chain.windows(2) {
        let from_type = &window[0];
        let to_type = &window[1];

        let mut from_type_str = from_type.into_token_stream().to_string();
        // Sanitize the type name for use as a Rust identifier (e.g. `Foo<Bar>` → `FooBar`)
        from_type_str.retain(|c| c.is_ascii_alphabetic() || c == '_');

        let mut to_type_str = to_type.into_token_stream().to_string();
        // Sanitize the type name for use as a Rust identifier (e.g. `Foo<Bar>` → `FooBar`)
        to_type_str.retain(|c| c.is_ascii_alphabetic() || c == '_');

        let transformer_name =
            format_ident!("__From_{}_To_{}Transformer", from_type_str, to_type_str);

        transformer_structs.push(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #transformer_name;
        });

        transformer_impls.push(quote! {
            impl version_core::version::VersionChangeTransformer for #transformer_name {
                type Input = #from_type;
                type Output = #to_type;

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
                id: <#to_type as version_core::version::VersionChange>::below_version(),
                changes: vec![::std::boxed::Box::new(#transformer_name)],
            });
        });
    }

    let version_ids = changes.iter().map(|ty| {
        quote! { <#ty as version_core::version::VersionChange>::below_version() }
    });

    let mut from_assertions = Vec::new();
    for window in chain.windows(2) {
        let from_type = &window[0];
        let to_type = &window[1];
        from_assertions.push(quote! {
            _assert_from::<#from_type, #to_type>();
        });
    }

    Ok(quote! {
        #(#transformer_structs)*
        #(#transformer_impls)*

        impl version_core::version::ChangeHistory for #change_history_type {
            type Head = #head;

            fn version_ids() -> ::std::vec::Vec<version_core::version::VersionId> {
                ::std::vec![#(#version_ids),*]
            }

            fn register(registry: &mut version_core::registry::ApiResponseResourceRegistry) {
                let version_ids = Self::version_ids();
                for window in version_ids.windows(2) {
                    if window[0] <= window[1] {
                        panic!(
                            "changes must be ordered newest-first by `below` version; got {:?} then {:?}",
                            window[0], window[1]
                        );
                    }
                }
                #(#register_entries)*
            }
        }

        impl #change_history_type {
            pub fn version_ids() -> ::std::vec::Vec<version_core::version::VersionId> {
                <Self as version_core::version::ChangeHistory>::version_ids()
            }

            pub fn register(registry: &mut version_core::registry::ApiResponseResourceRegistry) {
                <Self as version_core::version::ChangeHistory>::register(registry)
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

fn parse_changes_attr(attrs: &[Attribute]) -> syn::Result<Vec<Type>> {
    let changes_attr = attrs
        .iter()
        .find(|a| a.path().is_ident("changes"))
        .ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "missing #[changes(...)]")
        })?;
    let changes: Punctuated<Type, Token![,]> =
        changes_attr.parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated)?;
    Ok(changes.into_iter().collect())
}
