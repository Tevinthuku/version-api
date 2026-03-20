mod registry;
pub use registry::ResourceRegistry;
pub use registry::TransformDirection;

#[cfg(test)]
mod response_registry_tests {
    use std::any::TypeId;

    use crate::{
        registry::{ResourceRegistry, registry::TransformDirection},
        version::{ResourceType, Version, VersionChangeTransformer},
    };
    use version_id::VersionId;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct UserWithSingleAddress {
        #[allow(dead_code)]
        address: String,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct UserWithMultipleStringAddresses {
        addresses: Vec<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Address {
        location: String,
        country: Option<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct User {
        addresses: Vec<Address>,
    }

    struct CollapseAddressesToAddress;

    impl VersionChangeTransformer for CollapseAddressesToAddress {
        type Input = UserWithMultipleStringAddresses;
        type Output = UserWithSingleAddress;

        fn resource_type(&self) -> ResourceType {
            ResourceType::Response
        }

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

    impl VersionChangeTransformer for CollapseAddressesToListOfStr {
        type Input = User;
        type Output = UserWithMultipleStringAddresses;

        fn resource_type(&self) -> ResourceType {
            ResourceType::Response
        }

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
    fn test_transformation_works_for_legacy_version() {
        let mut registry = ResourceRegistry::default();

        let user_2 = User {
            addresses: vec![Address {
                location: "123 Main St".to_string(),
                country: Some("USA".to_string()),
            }],
        };

        registry.register(Version {
            id: VersionId::try_from("1.0.0").unwrap(),
            changes: vec![Box::new(CollapseAddressesToAddress)],
        });
        registry.register(Version {
            id: VersionId::try_from("2.0.0").unwrap(),
            changes: vec![Box::new(CollapseAddressesToListOfStr)],
        });

        let bytes = registry
            .transform(
                user_2,
                TransformDirection::DownForResponses {
                    user_version: VersionId::try_from("0.9.0").unwrap(),
                },
            )
            .expect("Transformation failed");

        let user_1: UserWithSingleAddress = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(user_1.address, "123 Main St USA".to_string());
    }

    #[test]
    fn test_latest_version_returns_head_unchanged() {
        let mut registry = ResourceRegistry::default();

        registry.register(Version {
            id: VersionId::try_from("1.0.0").unwrap(),
            changes: vec![Box::new(CollapseAddressesToAddress)],
        });
        registry.register(Version {
            id: VersionId::try_from("2.0.0").unwrap(),
            changes: vec![Box::new(CollapseAddressesToListOfStr)],
        });

        let user = User {
            addresses: vec![Address {
                location: "123 Main St".to_string(),
                country: Some("USA".to_string()),
            }],
        };

        let bytes = registry
            .transform(
                user,
                TransformDirection::DownForResponses {
                    user_version: VersionId::try_from("2.0.0").unwrap(),
                },
            )
            .expect("Transformation failed");
        let latest: User = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(latest.addresses.len(), 1);
        assert_eq!(latest.addresses[0].location, "123 Main St");
        assert_eq!(latest.addresses[0].country.as_deref(), Some("USA"));
    }
}

#[cfg(test)]
mod request_registry_tests {
    use std::any::TypeId;

    use crate::{
        registry::{ResourceRegistry, registry::TransformDirection},
        version::{ResourceType, Version, VersionChangeTransformer},
    };
    use version_id::VersionId;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct User {
        first_name: String,
        last_name: String,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct UserWithSingleNameField {
        name: String,
    }

    struct ExpandSingleNameToSplitNames;

    impl VersionChangeTransformer for ExpandSingleNameToSplitNames {
        type Input = UserWithSingleNameField;
        type Output = User;

        fn resource_type(&self) -> ResourceType {
            ResourceType::Request
        }

        fn description(&self) -> &str {
            "Older request payloads send `name`; latest expects `first_name` + `last_name`"
        }

        fn head_version(&self) -> TypeId {
            TypeId::of::<User>()
        }

        fn transform(
            &self,
            input: UserWithSingleNameField,
        ) -> Result<User, Box<dyn std::error::Error>> {
            let mut parts = input.name.splitn(2, ' ');
            let first_name = parts.next().unwrap_or_default().to_string();
            let last_name = parts.next().unwrap_or_default().to_string();

            Ok(User {
                first_name,
                last_name,
            })
        }
    }

    #[test]
    fn test_legacy_request_payload_is_upgraded_to_latest_model() {
        let mut registry = ResourceRegistry::default();

        // Legacy shape applies to versions below 2.0.0
        registry.register(Version {
            id: VersionId::try_from("2.0.0").unwrap(),
            changes: vec![Box::new(ExpandSingleNameToSplitNames)],
        });

        let legacy_request = UserWithSingleNameField {
            name: "Alice Doe".to_string(),
        };

        let bytes = registry
            .transform(
                legacy_request,
                TransformDirection::UpForRequests {
                    user_version: VersionId::try_from("1.0.0").unwrap(),
                    target_request_type: TypeId::of::<User>(),
                },
            )
            .expect("Request transformation failed");

        let latest_request: User = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(latest_request.first_name, "Alice");
        assert_eq!(latest_request.last_name, "Doe");
    }
}
