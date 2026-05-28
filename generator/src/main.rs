use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: Feature name is required.");
        eprintln!("Usage: cargo run --bin generator <feature_name>");
        std::process::exit(1);
    }

    let feature_name = &args[1];

    if !is_valid_snake_case(feature_name) {
        eprintln!("Error: Feature name must be in snake_case format (e.g., 'user_profile').");
        std::process::exit(1);
    }

    let camel_case = to_camel_case(feature_name);
    let kebab_case = to_kebab_case(feature_name);

    let target_dir_str = format!("src/features/{}", feature_name);
    let target_dir = Path::new(&target_dir_str);

    if target_dir.exists() {
        eprintln!(
            "Error: Directory '{}' already exists.",
            target_dir.display()
        );
        std::process::exit(1);
    }

    println!("Creating feature directory: {}...", target_dir.display());
    if let Err(e) = fs::create_dir_all(target_dir) {
        eprintln!("Failed to create directory: {}", e);
        std::process::exit(1);
    }

    // Write mod.rs
    let mod_content = "pub mod dto;\npub mod handler;\npub mod service;\n";
    if let Err(e) = write_file(&target_dir.join("mod.rs"), mod_content) {
        eprintln!("Failed to write mod.rs: {}", e);
        std::process::exit(1);
    }

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
    if let Err(e) = write_file(&target_dir.join("dto.rs"), &dto_content) {
        eprintln!("Failed to write dto.rs: {}", e);
        std::process::exit(1);
    }

    // Write handler.rs
    let handler_content = format!(
        r#"use axum::{{Json, extract::State, extract::Query, http::StatusCode}};
use sea_orm::DatabaseConnection;

use crate::{{
    infra::{{
        error::{{AppError, ErrorResponse}},
        extractor::ValidatedJson,
        pagination::{{PaginationParams, PaginatedResponse}},
    }},
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
    if let Err(e) = write_file(&target_dir.join("handler.rs"), &handler_content) {
        eprintln!("Failed to write handler.rs: {}", e);
        std::process::exit(1);
    }

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
    if let Err(e) = write_file(&target_dir.join("service.rs"), &service_content) {
        eprintln!("Failed to write service.rs: {}", e);
        std::process::exit(1);
    }

    // Register in src/features/mod.rs
    let mod_file_path = Path::new("src/features/mod.rs");
    if mod_file_path.exists() {
        let register_line = format!("pub mod {};", feature_name);
        match fs::read_to_string(mod_file_path) {
            Ok(content) => {
                if !content.contains(&register_line) {
                    println!("Registering module in {}...", mod_file_path.display());
                    let mut file = OpenOptions::new()
                        .append(true)
                        .open(mod_file_path)
                        .expect("Failed to open features mod.rs for appending");
                    if let Err(e) = writeln!(file, "pub mod {};", feature_name) {
                        eprintln!("Failed to append module registration: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read features mod.rs: {}", e);
            }
        }
    }

    println!("Feature '{}' generated successfully!", feature_name);
    println!("\nNext Steps:");
    println!("1. Register routes in 'src/routes/mod.rs':");
    println!(
        "   - Import the handler: 'use crate::features::{}::handler as {}_handler;'",
        feature_name, feature_name
    );
    println!("   - Define routes in 'create_router':");
    println!(
        "     let {}_routes = Router::new().route(\"/\", post({}_handler::create));",
        feature_name, feature_name
    );
    println!(
        "     ... .nest(\"/api/{}\", {}_routes)",
        kebab_case, feature_name
    );
    println!("2. Add the endpoints and DTOs to 'ApiDoc' in 'src/routes/mod.rs' for Swagger:");
    println!(
        "   - Add paths: 'crate::features::{}::handler::create'",
        feature_name
    );
    println!(
        "   - Add schemas: 'crate::features::{}::dto::Create{}Request', 'crate::features::{}::dto::{}Response'",
        feature_name, camel_case, feature_name, camel_case
    );
}

fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn is_valid_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_lowercase() {
            return false;
        }
    }
    for c in chars {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
            return false;
        }
    }
    true
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    s.replace('_', "-")
}
