# version-api

`version-api` is an experiment, inspired by Stripe's API versioning approach:
https://stripe.com/blog/api-versioning

The goal is to keep your latest request and response shapes as the source of truth, then:

- **Responses** are transformed _backwards_ (downgraded) for clients on older API versions.
- **Requests** are transformed _forwards_ (upgraded) from older client shapes into the latest model your handlers expect.

See `version-actix/examples` for complete, runnable examples.

### Response versioning quick example

```rust
use actix_web::{get, web, App, HttpServer, Result};
use serde::{Deserialize, Serialize};
use version_actix::{BaseActixVersionIdExtractor, VersionedJsonResponder};
use version_core::{
    ApiVersionId, ResponseChangeHistory, VersionChange, registry::ResourceRegistry,
};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, VersionChange)]
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
struct CurrentUserResponseHistory;

#[get("/user/{name}")]
async fn user_endpoint(name: web::Path<String>) -> Result<VersionedJsonResponder<CurrentUser>> {
    Ok(VersionedJsonResponder(CurrentUser {
        first_name: name.into_inner(),
        last_name: "Doe".to_string(),
    }))
}

let mut registry = ResourceRegistry::new();
CurrentUserResponseHistory::register(&mut registry).unwrap();

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

### Request versioning quick example

```rust
use actix_web::{post, web, App, HttpServer, Result};
use serde::{Deserialize, Serialize};
use version_actix::{BaseActixVersionIdExtractor, VersionedJsonRequest};
use version_core::{
    ApiVersionId, RequestChangeHistory, VersionChange, registry::ResourceRegistry,
};

#[derive(Serialize, Deserialize, VersionChange)]
#[description("The latest user model expects both first_name and last_name")]
struct CreateUserRequest {
    first_name: String,
    last_name: String,
}

// Same ApiVersion enum as above...

#[derive(Serialize, Deserialize, VersionChange)]
#[description = "Clients below 2.0.0 send a single name field"]
struct LegacyCreateUserRequest {
    name: String,
}

// For requests, the From direction is reversed: old shape → new shape
impl From<LegacyCreateUserRequest> for CreateUserRequest {
    fn from(obj: LegacyCreateUserRequest) -> Self {
        let mut parts = obj.name.splitn(2, ' ');
        Self {
            first_name: parts.next().unwrap_or_default().to_string(),
            last_name: parts.next().unwrap_or_default().to_string(),
        }
    }
}

#[derive(RequestChangeHistory)]
#[head(CreateUserRequest)]
#[changes(
    below(ApiVersion::V2_0_0) => LegacyCreateUserRequest,
)]
struct CreateUserRequestHistory;

#[post("/users")]
async fn create_user(user: VersionedJsonRequest<CreateUserRequest>) -> Result<web::Json<CreateUserRequest>> {
    Ok(web::Json(user.into_inner()))
}

let mut registry = ResourceRegistry::new();
CreateUserRequestHistory::register(&mut registry).unwrap();

// ... same extractor and HttpServer setup as the response example
```

## Workspace crates

- `version-id`: `VersionId` type and validation interface.
- `version-api-macros`: derive macros (`ApiVersionId`, `ResponseChangeHistory`, `RequestChangeHistory`, `VersionChange`).
- `version-core`: version change traits and transformation registry.
- `version-actix`: Actix integration (`VersionedJsonResponder`, `VersionedJsonRequest`, version extractors).

## Design overview

The system follows a "latest-first + transform" design that works in both directions:

**Responses (downgrade):**

1. Handler builds the latest response model.
2. A version ID is resolved from the request (header, middleware, request extensions, etc.).
3. `version-core` applies registered transforms from newest to oldest until the target version boundary is reached.
4. The downgraded payload is returned to the client.

**Requests (upgrade):**

1. The incoming request body is deserialized.
2. A version ID is resolved from the request.
3. `version-core` applies registered transforms from oldest to newest, upgrading the body to the latest shape.
4. The handler receives the latest model, regardless of which version the client sent.

This design keeps current code paths simple while isolating legacy behavior in explicit version change types.

## Core concepts

### `VersionId`

`version-id` wraps semantic version semantics in `VersionId` and exposes a validator trait (`VersionIdValidator`) used by framework integrations.

### Version changes

In `version-core`:

- `VersionChangeTransformer` defines a typed transformation (`Input -> Output`).
- `ErasedVersionChangeTransformer` type-erases transformers so heterogeneous changes can be stored in one registry.
- `ResourceRegistry` stores changes per resource type and version, separated by direction (request vs response), then applies them in the appropriate order.

### Derive macros

`version-api-macros` removes boilerplate:

- `#[derive(ApiVersionId)]` on version enums with `#[version("x.y.z")]`.
- `#[derive(VersionChange)]` on historical DTOs.
- `#[derive(ResponseChangeHistory)]` to declare and register response downgrade chains.
- `#[derive(RequestChangeHistory)]` to declare and register request upgrade chains.

### Actix integration

`version-actix` provides:

- `VersionedJsonResponder<T>`: serializes and conditionally transforms outgoing JSON responses.
- `VersionedJsonRequest<T>`: deserializes and conditionally transforms incoming JSON request bodies.
- `ActixVersionIdExtractor`: a trait that defines how to resolve the request's API version in Actix.
- `BaseActixVersionIdExtractor`: default header-based extractor.

## Current scope

- Response payload versioning (JSON downgrade transformations) is implemented.
- Request body versioning (JSON upgrade transformations) is implemented.
- Version extraction is pluggable in Actix.

## Roadmap

Planned next areas:

1. Improve error handling
