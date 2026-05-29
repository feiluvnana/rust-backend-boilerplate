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
    let mod_content = "pub mod dto;\npub mod handler;\npub mod router;\npub mod service;\n";
    let _ = write_file(&target_dir.join("mod.rs"), mod_content);

    // Write dto.rs
    let dto_content = format!(
        r#"#[allow(unused_imports)]
use serde::{{Deserialize, Serialize}};
#[allow(unused_imports)]
use utoipa::ToSchema;
#[allow(unused_imports)]
use validator::Validate;
"#
    );
    let _ = write_file(&target_dir.join("dto.rs"), &dto_content);

    // Write handler.rs
    let handler_content = format!(
        r#"#[allow(unused_imports)]
use axum::{{extract::State, http::StatusCode, Json}};
#[allow(unused_imports)]
use sea_orm::DatabaseConnection;

#[allow(unused_imports)]
use crate::{{
    infra::error::AppError,
    features::{FeatureName}::service::{CamelName}Service,
}};
"#,
        FeatureName = feature_name,
        CamelName = camel_case
    );
    let _ = write_file(&target_dir.join("handler.rs"), &handler_content);

    // Write service.rs
    let service_content = format!(
        r#"#[allow(unused_imports)]
use sea_orm::DatabaseConnection;

pub struct {CamelName}Service;

impl {CamelName}Service {{
}}
"#,
        CamelName = camel_case
    );
    let _ = write_file(&target_dir.join("service.rs"), &service_content);

    // Write router.rs
    let routes_content = format!(
        r#"use axum::Router;

use crate::routes::AppState;

pub fn router() -> Router<AppState> {{
    Router::new()
}}
"#
    );
    let _ = write_file(&target_dir.join("router.rs"), &routes_content);

    register_feature_in_mod(feature_name);
    register_routes_in_mod(feature_name, &kebab_case);

    println!("Feature '{}' generated successfully!", feature_name);
}
