# version-api

`version-api` is an experiment, inspired by Stripe's API versioning approach:
https://stripe.com/blog/api-versioning

The goal is to keep your latest response shape as the source of truth, then transform it backwards for older API versions when needed.

Request-side versioning (body/params) is coming soonish.

See `version-actix/examples` for a complete, runnable example.

### Actix quick example

```rust
use actix_web::{get, web, App, HttpServer, Result};
use serde::Serialize;
use version_actix::{BaseActixVersionIdExtractor, VersionedJsonResponder};
use version_core::{
    ApiVersionId, RequestChangeHistory, VersionChange, registry::ResourceRegistry,
};

#[derive(Serialize)]
struct CurrentUser {
    first_name: String,
    last_name: String,
}

#[derive(ApiVersionId)]
enum ApiVersion {
    #[version("2.0.0")]
    V2_0_0,
    #[version("1.0.0")]
    V1_0_0,
}

#[derive(Serialize, VersionChange)]
#[description = "Clients below 2.0.0 expect a single `name` field"]
struct LegacyUserName {
    name: String,
}

impl From<CurrentUser> for LegacyUserName {
    fn from(user: CurrentUser) -> Self {
        Self {
            name: format!("{} {}", user.first_name, user.last_name),
        }
    }
}

#[derive(ResponseChangeHistory)]
#[head(CurrentUser)]
#[changes(
    below(ApiVersion::V2_0_0) => LegacyUserName,
)]
struct CurrentUserResponseHistoryVersions;

#[get("/user/{name}")]
async fn user_endpoint(name: web::Path<String>) -> Result<VersionedJsonResponder<CurrentUser>> {
    Ok(VersionedJsonResponder(CurrentUser {
        first_name: name.into_inner(),
        last_name: "Doe".to_string(),
    }))
}

let mut registry = ResourceRegistry::new();
CurrentUserResponseHistoryVersions::register(&mut registry).unwrap();

let version_id_extractor = web::Data::from(BaseActixVersionIdExtractor::header_extractor(
    "X-API-Version".to_string(),
    ApiVersion::validator(),
));

let registry = web::Data::new(registry);
HttpServer::new(move || {
    App::new()
        .app_data(registry.clone())
        .app_data(version_id_extractor.clone())
        .service(user_endpoint)
});
```

## Workspace crates

- `version-id`: `VersionId` type and validation interface.
- `version-api-macros`: derive macros (`ApiVersionId`, `RequestChangeHistory`, `VersionChange`).
- `version-core`: version change traits and transformation registry.
- `version-actix`: Actix integration (`VersionedJsonResponder` + version extractors).

## Design overview

The system follows a "latest-first + rollback transforms" design:

1. Handler builds the latest response model.
2. A request version ID is resolved (header, middleware, request extensions, etc.).
3. `version-core` applies registered transforms from newest to oldest until the target version boundary is reached.
4. The transformed payload is returned.

This design keeps current code paths simple while isolating legacy behavior in explicit version change types.

## Core concepts

### `VersionId`

`version-id` wraps semantic version semantics in `VersionId` and exposes a validator trait (`VersionIdValidator`) used by framework integrations.

### Version changes

In `version-core`:

- `VersionChangeTransformer` defines a typed transformation (`Input -> Output`).
- `ErasedVersionChangeTransformer` type-erases transformers so heterogeneous changes can be stored in one registry.
- `ResourceRegistry` stores changes per response type and version, then applies them in descending version order.

### Derive macros

`version-api-macros` removes boilerplate:

- `#[derive(ApiVersionId)]` on version enums with `#[version("x.y.z")]`.
- `#[derive(VersionChange)]` on historical DTOs.
- `#[derive(ResponseChangeHistory)]` to declare and register downgrade chains.

### Actix integration

`version-actix` provides:

- `VersionedJsonResponder<T>`: serializes and conditionally transforms outgoing JSON.
- `ActixVersionIdExtractor`: a trait that defines how to resolve the request’s API version in Actix.
- `BaseActixVersionIdExtractor`: default header-based extractor.

## Current scope

- Response payload versioning (JSON transformations) is implemented.
- Version extraction is pluggable in Actix.

## Roadmap

Planned next areas: (Once I get the time to actually do it)

1. Request body versioning (input shape compatibility transforms).
2. Request params versioning (path/query/header normalization across versions).
