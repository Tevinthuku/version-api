mod response;

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::{
        registry::response::ApiResponseResourceRegistry,
        version::{Version, VersionChangeSetTransformer, VersionId},
    };

    struct Userv0 {
        #[allow(dead_code)]
        address: String,
    }

    #[derive(Debug)]
    struct Userv1 {
        addresses: Vec<String>,
    }

    struct Address {
        location: String,
        country: Option<String>,
    }

    struct Userv2 {
        addresses: Vec<Address>,
    }

    struct CollapseAddressesToAddress;

    impl VersionChangeSetTransformer for CollapseAddressesToAddress {
        type Input = Userv1;
        type Output = Userv0;

        fn description(&self) -> &str {
            "We replaced address with addresses because we now support multiple addresses per user"
        }

        fn head_version(&self) -> TypeId {
            TypeId::of::<Userv2>()
        }

        fn transform(&self, input: Userv1) -> Result<Userv0, Box<dyn std::error::Error>> {
            Ok(Userv0 {
                address: input.addresses.first().cloned().unwrap_or_default(),
            })
        }
    }

    struct CollapseAddressesToListOfStr;

    impl VersionChangeSetTransformer for CollapseAddressesToListOfStr {
        type Input = Userv2;
        type Output = Userv1;

        fn head_version(&self) -> TypeId {
            TypeId::of::<Userv2>()
        }

        fn description(&self) -> &str {
            "Addresses will now render a list of objects {location: string, country: string | null} instead of a list of strings"
        }

        fn transform(&self, input: Userv2) -> Result<Userv1, Box<dyn std::error::Error>> {
            Ok(Userv1 {
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

        let user_2 = Userv2 {
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

        let user_1 = transformed.downcast::<Userv1>().unwrap();

        println!("u: {:?}", user_1);
    }
}
