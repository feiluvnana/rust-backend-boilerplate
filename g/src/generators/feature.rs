use std::fs;
use std::path::Path;

use crate::registrars::features::register_feature_in_mod;
use crate::registrars::routes::register_routes_in_mod;
use crate::utils::{to_camel_case, to_kebab_case, write_file};

pub fn generate(feature_name: &str) {
    let camel_case = to_camel_case(feature_name);
    let kebab_case = to_kebab_case(feature_name);

    let target_dir_str = format!("src/features/{}", feature_name);
    let target_dir = Path::new(&target_dir_str);

    if target_dir.exists() {
        eprintln!("Error: Directory '{}' already exists.", target_dir.display());
        std::process::exit(1);
    }

    println!("Creating feature directory: {}...", target_dir.display());
    if let Err(e) = fs::create_dir_all(target_dir) {
        eprintln!("Failed to create directory: {}", e);
        std::process::exit(1);
    }

    // Write mod.rs
    let mod_content = "pub mod dto;\npub mod handler;\npub mod router;\npub mod service;\n";
    let _ = write_file(&target_dir.join("mod.rs"), mod_content);

    // Write dto.rs
    let dto_content = format!(
        r#"use serde::{{Deserialize, Serialize}};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema, Clone)]
pub struct Create{Request}Request {{
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct {Response}Response {{
    pub id: i32,
    pub name: String,
}}
"#,
        Request = camel_case,
        Response = camel_case
    );
    let _ = write_file(&target_dir.join("dto.rs"), &dto_content);

    // Write handler.rs
    let handler_content = format!(
        r#"use axum::{{Json, extract::State, http::StatusCode}};
use sea_orm::DatabaseConnection;

use crate::{{
    infra::error::{{AppError, ErrorResponse}},
    extractors::ValidatedJson,
    features::{FeatureName}::{{
        dto::{{Create{CamelName}Request, {CamelName}Response}},
        service::{CamelName}Service,
    }},
}};

#[utoipa::path(
    post,
    path = "/api/{KebabName}",
    request_body = Create{CamelName}Request,
    responses(
        (status = 201, description = "Created successfully", body = {CamelName}Response),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn create(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<Create{CamelName}Request>,
) -> Result<(StatusCode, Json<{CamelName}Response>), AppError> {{
    let result = {CamelName}Service::create_item(&db, &payload.name).await?;

    Ok((
        StatusCode::CREATED,
        Json({CamelName}Response {{
            id: result,
            name: payload.name,
        }}),
    ))
}}
"#,
        FeatureName = feature_name,
        CamelName = camel_case,
        KebabName = kebab_case
    );
    let _ = write_file(&target_dir.join("handler.rs"), &handler_content);

    // Write service.rs
    let service_content = format!(
        r#"use sea_orm::DatabaseConnection;
use crate::infra::error::AppError;

pub struct {CamelName}Service;

impl {CamelName}Service {{
    /// Example service method
    pub async fn create_item(db: &DatabaseConnection, _name: &str) -> Result<i32, AppError> {{
        // Implement database logic here
        Ok(1)
    }}
}}
"#,
        CamelName = camel_case
    );
    let _ = write_file(&target_dir.join("service.rs"), &service_content);

    // Write router.rs
    let routes_content = format!(
        r#"use axum::{{
    routing::post,
    Router,
}};

use crate::routes::AppState;
use super::handler as handler;

pub fn router() -> Router<AppState> {{
    Router::new()
        .route("/", post(handler::create))
}}
"#
    );
    let _ = write_file(&target_dir.join("router.rs"), &routes_content);

    register_feature_in_mod(feature_name);
    register_routes_in_mod(feature_name, &kebab_case);

    println!("Feature '{}' generated successfully!", feature_name);
}
