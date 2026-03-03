use version_core::{
    ChangeHistory, VersionChange, registry::ApiResponseResourceRegistry, version::VersionId,
};

#[derive(Debug, Clone)]
struct Address {
    location: String,
}

#[derive(Debug, Clone)]
struct User {
    name: String,
    addresses: Vec<Address>,
}

#[derive(Debug, VersionChange)]
#[version(below = "1.0.0")]
#[description = "Legacy users expect one address string"]
#[allow(dead_code)]
struct CollapseUserAddressToSingleString {
    name: String,
    address: String,
}

#[derive(Debug, VersionChange)]
#[version(below = "2.0.0")]
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
            // the users on less than v1 expect a single address, so its fine to just take the first one
            address: user.addresses.first().cloned().unwrap_or_default(),
        }
    }
}

#[derive(ChangeHistory)]
#[head(User)]
#[changes(CollapseUserAddressesToStrings, CollapseUserAddressToSingleString)]
struct UserResponseHistoryVersions;

fn main() {
    let mut registry = ApiResponseResourceRegistry::default();
    UserResponseHistoryVersions::register(&mut registry);

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

    let transformed = registry
        .transform(user.clone(), VersionId::from("1.0.0"))
        .unwrap();
    let user_with_string_addresses = transformed
        .downcast::<CollapseUserAddressesToStrings>()
        .unwrap();
    assert_eq!(
        user_with_string_addresses.addresses,
        vec!["123 Main St", "456 Main St"]
    );

    let transformed = registry
        .transform(user.clone(), VersionId::from("0.9.0"))
        .unwrap();
    let user_with_single_address = transformed
        .downcast::<CollapseUserAddressToSingleString>()
        .unwrap();
    assert_eq!(user_with_single_address.address, "123 Main St".to_string());
}
