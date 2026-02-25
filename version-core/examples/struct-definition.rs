use version_core::{VersionedResponse, registry::ApiResponseResourceRegistry, version::VersionId};
#[derive(Debug)]
struct UserWithNameOnly {
    name: String,
}

#[derive(Debug)]
struct UserWithSingleAddress {
    name: String,
    address: String,
}

#[derive(Debug)]
struct UserWithStringAddresses {
    name: String,
    addresses: Vec<String>,
}

#[derive(Debug)]
struct Address {
    location: String,
}

#[derive(Debug, VersionedResponse)]
#[response(changed_in(
    "v3" => UserWithStringAddresses,
    "v2" => UserWithSingleAddress,
    "v1" => UserWithNameOnly,
))]
struct User {
    name: String,
    addresses: Vec<Address>,
}

impl From<User> for UserWithStringAddresses {
    fn from(user: User) -> Self {
        println!("Transforming from User to UserWithSingleAddress");
        Self {
            name: user.name,
            addresses: user.addresses.into_iter().map(|a| a.location).collect(),
        }
    }
}

impl From<UserWithStringAddresses> for UserWithSingleAddress {
    fn from(user: UserWithStringAddresses) -> Self {
        println!("Transforming from UserWithStringAddresses to UserWithSingleAddress");
        Self {
            name: user.name,
            address: user.addresses.first().cloned().unwrap_or_default(),
        }
    }
}

impl From<UserWithSingleAddress> for UserWithNameOnly {
    fn from(user: UserWithSingleAddress) -> Self {
        println!("Transforming from UserWithSingleAddress to UserWithNameOnly");
        Self { name: user.name }
    }
}

fn main() {
    let mut registry = ApiResponseResourceRegistry::default();
    User::register_versions(&mut registry);

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

    let transformed = registry.transform(user, VersionId::from("v1")).unwrap();
    let user_with_single_address = transformed.downcast::<UserWithNameOnly>().unwrap();
    println!("User with single address: {:?}", user_with_single_address);
}
