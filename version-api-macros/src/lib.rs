mod derive_api_version_id;
mod derive_change_history;
mod derive_version_change;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransformDirection {
    Request,
    Response,
}

#[proc_macro_derive(RequestChangeHistory, attributes(head, changes))]
pub fn request_change_history_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_change_history::change_history_derive_impl(input, TransformDirection::Request)
}

#[proc_macro_derive(ResponseChangeHistory, attributes(head, changes))]
pub fn response_change_history_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_change_history::change_history_derive_impl(input, TransformDirection::Response)
}

#[proc_macro_derive(VersionChange, attributes(version, description))]
pub fn version_change_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_version_change::version_change_derive_impl(input)
}

#[proc_macro_derive(ApiVersionId, attributes(version))]
pub fn api_version_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_api_version_id::api_version_derive_impl(input)
}
