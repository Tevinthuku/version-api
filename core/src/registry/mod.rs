mod response;

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::{
        registry::response::ApiResponseResourceRegistry,
        version::{Version, VersionChangeSetTransformer, VersionId},
    };

    struct UserWithSingleAddress {
        #[allow(dead_code)]
        address: String,
    }

    #[derive(Debug)]
    struct UserWithMultipleStringAddresses {
        addresses: Vec<String>,
    }

    struct Address {
        location: String,
        country: Option<String>,
    }

    struct User {
        addresses: Vec<Address>,
    }

    struct CollapseAddressesToAddress;

    impl VersionChangeSetTransformer for CollapseAddressesToAddress {
        type Input = UserWithMultipleStringAddresses;
        type Output = UserWithSingleAddress;

        fn description(&self) -> &str {
            "We replaced address with addresses because we now support multiple addresses per user"
        }

        fn head_version(&self) -> TypeId {
            TypeId::of::<User>()
        }

        fn transform(
            &self,
            input: UserWithMultipleStringAddresses,
        ) -> Result<UserWithSingleAddress, Box<dyn std::error::Error>> {
            Ok(UserWithSingleAddress {
                address: input.addresses.first().cloned().unwrap_or_default(),
            })
        }
    }

    struct CollapseAddressesToListOfStr;

    impl VersionChangeSetTransformer for CollapseAddressesToListOfStr {
        type Input = User;
        type Output = UserWithMultipleStringAddresses;

        fn head_version(&self) -> TypeId {
            TypeId::of::<User>()
        }

        fn description(&self) -> &str {
            "Addresses will now render a list of objects {location: string, country: string | null} instead of a list of strings"
        }

        fn transform(
            &self,
            input: User,
        ) -> Result<UserWithMultipleStringAddresses, Box<dyn std::error::Error>> {
            Ok(UserWithMultipleStringAddresses {
                addresses: input
                    .addresses
                    .into_iter()
                    .map(|a| format!("{} {}", a.location, a.country.unwrap_or_default()))
                    .collect(),
            })
        }
    }

    #[test]
    fn test_transformation_works() {
        let mut registry = ApiResponseResourceRegistry::default();

        let user_2 = User {
            addresses: vec![Address {
                location: "123 Main St".to_string(),
                country: Some("USA".to_string()),
            }],
        };

        registry.register(Version {
            id: VersionId::from("v1"),
            changes: vec![Box::new(CollapseAddressesToAddress)],
        });
        registry.register(Version {
            id: VersionId::from("v2"),
            changes: vec![Box::new(CollapseAddressesToListOfStr)],
        });

        let transformed = registry
            .transform(user_2, VersionId::from("v1"))
            .expect("Transformation failed");

        let user_1 = transformed
            .downcast::<UserWithMultipleStringAddresses>()
            .unwrap();

        assert_eq!(user_1.addresses, vec!["123 Main St USA".to_string()]);
    }
}
