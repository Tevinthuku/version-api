use version_core::{
    ChangeLog, ChangeSet, registry::ApiResponseResourceRegistry, version::VersionId,
};

#[derive(Debug)]
struct Address {
    location: String,
}

#[derive(Debug)]
struct User {
    name: String,
    addresses: Vec<Address>,
}

#[derive(Debug, ChangeSet)]
#[version(below = "1.0.0")]
#[description = "Legacy users expect one address string"]
#[allow(dead_code)]
struct CollapseUserAddressToSingleString {
    name: String,
    address: String,
}

#[derive(Debug, ChangeSet)]
#[version(below = "2.0.0")]
#[description = "Users before 2.0.0 expect addresses as plain strings"]
struct CollapseUserAddressesToStrings {
    name: String,
    addresses: Vec<String>,
}

impl From<User> for CollapseUserAddressesToStrings {
    fn from(user: User) -> Self {
        println!("Transforming User -> CollapseUserAddressesToStrings");
        Self {
            name: user.name,
            addresses: user.addresses.into_iter().map(|a| a.location).collect(),
        }
    }
}

impl From<CollapseUserAddressesToStrings> for CollapseUserAddressToSingleString {
    fn from(user: CollapseUserAddressesToStrings) -> Self {
        println!(
            "Transforming CollapseUserAddressesToStrings -> CollapseUserAddressToSingleString"
        );
        Self {
            name: user.name,
            address: user.addresses.first().cloned().unwrap_or_default(),
        }
    }
}

#[derive(ChangeLog)]
#[head(User)]
#[changes(CollapseUserAddressesToStrings, CollapseUserAddressToSingleString)]
struct UserChanges;

fn main() {
    let mut registry = ApiResponseResourceRegistry::default();
    UserChanges::register(&mut registry);

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

    let transformed = registry.transform(user, VersionId::from("1.0.0")).unwrap();
    let legacy_user = transformed
        .downcast::<CollapseUserAddressesToStrings>()
        .unwrap();
    println!("Legacy user: {:?}", legacy_user);
}
