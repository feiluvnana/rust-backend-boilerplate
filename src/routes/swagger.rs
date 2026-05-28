use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::features::health::handler::health,
        crate::features::health::handler::readiness,
    ),
    components(schemas(
        crate::features::health::handler::HealthStatus,
        crate::infra::error::ErrorResponse,
        crate::infra::error::FieldError,
        crate::infra::pagination::PageMeta,
    ))
)]
pub struct ApiDoc;
