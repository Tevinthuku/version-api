use version_core::{
    ApiVersionId, ChangeHistory, VersionChange, registry::ApiResponseResourceRegistry,
};

#[derive(ApiVersionId)]
pub enum MyApiVersions {
    #[version("10.0.0")]
    V10_0_0,
    #[version("2.0.0")]
    V2_0_0,
    #[version("1.0.0")]
    V1_0_0,
    #[version("0.9.0")]
    V0_9_0,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Address {
    location: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    name: String,
    addresses: Vec<Address>,
}

#[derive(Debug, VersionChange, serde::Serialize, serde::Deserialize)]
#[description = "Legacy users expect one address string"]
#[allow(dead_code)]
struct CollapseUserAddressToSingleString {
    name: String,
    address: String,
}

#[derive(Debug, VersionChange, serde::Serialize, serde::Deserialize)]
#[description = "Users before 2.0.0 expect addresses as plain strings"]
struct CollapseUserAddressesToStrings {
    name: String,
    addresses: Vec<String>,
}

impl From<User> for CollapseUserAddressesToStrings {
    fn from(user: User) -> Self {
        Self {
            name: user.name,
            addresses: user.addresses.into_iter().map(|a| a.location).collect(),
        }
    }
}

impl From<CollapseUserAddressesToStrings> for CollapseUserAddressToSingleString {
    fn from(user: CollapseUserAddressesToStrings) -> Self {
        Self {
            name: user.name,
            // the users on less than v1 expect a single address, so it's fine to just take the first one
            address: user.addresses.first().cloned().unwrap_or_default(),
        }
    }
}

#[derive(ChangeHistory)]
#[head(User)]
#[changes(
    below(MyApiVersions::V2_0_0) => CollapseUserAddressesToStrings,
    below(MyApiVersions::V1_0_0) => CollapseUserAddressToSingleString,
)]
struct UserResponseHistoryVersions;

fn main() {
    let mut registry = ApiResponseResourceRegistry::default();
    UserResponseHistoryVersions::register(&mut registry).unwrap();

    let user = User {
        name: "John Doe".to_string(),
        addresses: vec![
            Address {
                location: "123 Main St".to_string(),
            },
            Address {
                location: "456 Main St".to_string(),
            },
        ],
    };

    let bytes = registry
        .transform(user.clone(), MyApiVersions::V1_0_0.as_version_id())
        .unwrap();
    let user_with_string_addresses: CollapseUserAddressesToStrings =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        user_with_string_addresses.addresses,
        vec!["123 Main St", "456 Main St"]
    );

    let bytes = registry
        .transform(user.clone(), MyApiVersions::V0_9_0.as_version_id())
        .unwrap();
    let user_with_single_address: CollapseUserAddressToSingleString =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(user_with_single_address.address, "123 Main St".to_string());
}
