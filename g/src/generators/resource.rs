use std::fs;
use std::path::Path;

use crate::registrars::features::register_feature_in_mod;
use crate::registrars::routes::register_routes_in_mod;
use crate::registrars::swagger::register_in_swagger;
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

    println!("Creating resource directory: {}...", target_dir.display());
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
pub struct Create{CamelName}Request {{
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}}

#[derive(Debug, Deserialize, Validate, ToSchema, Clone)]
pub struct Update{CamelName}Request {{
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct {CamelName}Response {{
    pub id: i32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}}

impl From<crate::db::models::{SnakeName}::Model> for {CamelName}Response {{
    fn from(model: crate::db::models::{SnakeName}::Model) -> Self {{
        Self {{
            id: model.id,
            name: model.name,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }}
    }}
}}
"#,
        CamelName = camel_case,
        SnakeName = feature_name
    );
    let _ = write_file(&target_dir.join("dto.rs"), &dto_content);

    // Write service.rs
    let service_content = format!(
        r#"use sea_orm::{{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, Set,
}};
use crate::{{
    db::models::{SnakeName} as {SnakeName}_model,
    infra::error::AppError,
}};

pub struct {CamelName}Service;

impl {CamelName}Service {{
    pub async fn create(
        db: &DatabaseConnection,
        name: &str,
    ) -> Result<{SnakeName}_model::Model, AppError> {{
        let active_model = {SnakeName}_model::ActiveModel {{
            name: Set(name.to_owned()),
            ..Default::default()
        }};
        let model = active_model.insert(db).await?;
        Ok(model)
    }}

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<{SnakeName}_model::Model>, AppError> {{
        let model = {SnakeName}_model::Entity::find_by_id(id).one(db).await?;
        Ok(model)
    }}

    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<{SnakeName}_model::Model>, u64), AppError> {{
        let paginator = {SnakeName}_model::Entity::find()
            .order_by_desc({SnakeName}_model::Column::Id)
            .paginate(db, per_page);

        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page - 1).await?;
        Ok((items, total))
    }}

    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        name: &str,
    ) -> Result<{SnakeName}_model::Model, AppError> {{
        let model = {SnakeName}_model::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("{{}} not found", "{CamelName}")))?;

        let mut active_model: {SnakeName}_model::ActiveModel = model.into();
        active_model.name = Set(name.to_owned());
        active_model.updated_at = Set(chrono::DateTime::<chrono::FixedOffset>::from(chrono::Utc::now()));

        let updated = active_model.update(db).await?;
        Ok(updated)
    }}

    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), AppError> {{
        let result = {SnakeName}_model::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {{
            return Err(AppError::NotFound(format!("{{}} not found", "{CamelName}")));
        }}
        Ok(())
    }}
}}
"#,
        CamelName = camel_case,
        SnakeName = feature_name
    );
    let _ = write_file(&target_dir.join("service.rs"), &service_content);

    // Write handler.rs
    let handler_content = format!(
        r#"use axum::{{
    extract::{{Path, Query, State}},
    http::StatusCode,
    Json,
}};
use sea_orm::DatabaseConnection;

use crate::{{
    features::{SnakeName}::{{
        dto::{{Create{CamelName}Request, Update{CamelName}Request, {CamelName}Response}},
        service::{CamelName}Service,
    }},
    infra::{{
        error::{{AppError, ErrorResponse}},
        pagination::{{PaginatedResponse, PaginationParams}},
    }},
    extractors::ValidatedJson,
}};

#[utoipa::path(
    post,
    path = "/api/{KebabName}",
    request_body = Create{CamelName}Request,
    responses(
        (status = 201, description = "Created successfully", body = {CamelName}Response),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 409, description = "Conflict", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn create(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<Create{CamelName}Request>,
) -> Result<(StatusCode, Json<{CamelName}Response>), AppError> {{
    let result = {CamelName}Service::create(&db, &payload.name).await?;
    Ok((StatusCode::CREATED, Json({CamelName}Response::from(result))))
}}

#[utoipa::path(
    get,
    path = "/api/{KebabName}",
    params(PaginationParams),
    responses(
        (status = 200, description = "List retrieved successfully", body = PaginatedResponse<{CamelName}Response>),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    Query(params): Query<PaginationParams>,
) -> Result<(StatusCode, Json<PaginatedResponse<{CamelName}Response>>), AppError> {{
    let page = params.page();
    let per_page = params.per_page();
    let (items, total) = {CamelName}Service::list(&db, page, per_page).await?;
    let data = items.into_iter().map({CamelName}Response::from).collect();
    Ok((
        StatusCode::OK,
        Json(PaginatedResponse::new(data, page, per_page, total)),
    ))
}}

#[utoipa::path(
    get,
    path = "/api/{KebabName}/{{id}}",
    params(
        ("id" = i32, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Retrieved successfully", body = {CamelName}Response),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn get_by_id(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<{CamelName}Response>), AppError> {{
    let model = {CamelName}Service::find_by_id(&db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("{{}} not found", "{CamelName}")))?;
    Ok((StatusCode::OK, Json({CamelName}Response::from(model))))
}}

#[utoipa::path(
    put,
    path = "/api/{KebabName}/{{id}}",
    request_body = Update{CamelName}Request,
    params(
        ("id" = i32, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Updated successfully", body = {CamelName}Response),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn update(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<Update{CamelName}Request>,
) -> Result<(StatusCode, Json<{CamelName}Response>), AppError> {{
    let result = {CamelName}Service::update(&db, id, &payload.name).await?;
    Ok((StatusCode::OK, Json({CamelName}Response::from(result))))
}}

#[utoipa::path(
    delete,
    path = "/api/{KebabName}/{{id}}",
    params(
        ("id" = i32, Path, description = "Resource ID")
    ),
    responses(
        (status = 204, description = "Deleted successfully"),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn delete(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {{
    {CamelName}Service::delete(&db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}}
"#,
        CamelName = camel_case,
        SnakeName = feature_name,
        KebabName = kebab_case
    );
    let _ = write_file(&target_dir.join("handler.rs"), &handler_content);

    // Write router.rs
    let routes_content = format!(
        r#"use axum::{{
    routing::get,
    Router,
}};

use crate::routes::AppState;
use super::handler as handler;

pub fn router() -> Router<AppState> {{
    Router::new()
        .route("/", get(handler::list).post(handler::create))
        .route(
            "/{{id}}",
            get(handler::get_by_id)
                .put(handler::update)
                .delete(handler::delete),
        )
}}
"#
    );
    let _ = write_file(&target_dir.join("router.rs"), &routes_content);

    register_feature_in_mod(feature_name);
    register_routes_in_mod(feature_name, &kebab_case);
    register_in_swagger(feature_name, &camel_case);

    println!("Resource '{}' generated successfully!", feature_name);
}
