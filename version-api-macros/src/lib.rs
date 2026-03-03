mod api_version;
mod version_history;

#[proc_macro_derive(ChangeHistory, attributes(head, changes))]
pub fn change_history_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    version_history::change_history_derive_impl(input)
}

#[proc_macro_derive(VersionChange, attributes(version, description))]
pub fn version_change_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    version_history::version_change_derive_impl(input)
}

#[proc_macro_derive(ApiVersion, attributes(version))]
pub fn api_version_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    api_version::api_version_derive_impl(input)
}
