use std::fs;
use std::path::Path;

pub fn register_in_swagger(feature_name: &str, camel_case: &str) {
    let swagger_file_path = Path::new("src/routes/swagger.rs");
    if !swagger_file_path.exists() {
        return;
    }

    if let Ok(mut content) = fs::read_to_string(swagger_file_path) {
        // Paths
        let paths_marker = "paths(";
        if let Some(pos) = content.find(paths_marker) {
            let insert_pos = pos + paths_marker.len();
            let new_paths = format!(
                "\n        crate::features::{SnakeName}::handler::create,\n        crate::features::{SnakeName}::handler::list,\n        crate::features::{SnakeName}::handler::get_by_id,\n        crate::features::{SnakeName}::handler::update,\n        crate::features::{SnakeName}::handler::delete,",
                SnakeName = feature_name
            );
            if !content.contains(&format!(
                "crate::features::{}::handler::create",
                feature_name
            )) {
                content.insert_str(insert_pos, &new_paths);
            }
        }

        // Schemas
        let schemas_marker = "components(schemas(";
        if let Some(pos) = content.find(schemas_marker) {
            let insert_pos = pos + schemas_marker.len();
            let new_schemas = format!(
                "\n        crate::features::{SnakeName}::dto::Create{CamelName}Request,\n        crate::features::{SnakeName}::dto::Update{CamelName}Request,\n        crate::features::{SnakeName}::dto::{CamelName}Response,\n        crate::infra::pagination::PaginatedResponse<crate::features::{SnakeName}::dto::{CamelName}Response>,",
                SnakeName = feature_name,
                CamelName = camel_case
            );
            if !content.contains(&format!(
                "crate::features::{}::dto::Create{}Request",
                feature_name, camel_case
            )) {
                content.insert_str(insert_pos, &new_schemas);
            }
        }

        if let Err(e) = fs::write(swagger_file_path, content) {
            eprintln!("Failed to register components in swagger.rs: {}", e);
        }
    }
}
