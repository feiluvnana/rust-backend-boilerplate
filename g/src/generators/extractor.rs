use std::path::Path;

use crate::registrars::extractors::register_extractor_in_mod;
use crate::utils::write_file;

pub fn generate(name: &str) {
    let target_file_str = format!("src/extractors/{}.rs", name);
    let target_file = Path::new(&target_file_str);

    if target_file.exists() {
        eprintln!("Error: Extractor file '{}' already exists.", target_file.display());
        std::process::exit(1);
    }

    let camel_case = crate::utils::to_camel_case(name);

    let extractor_content = format!(
        r#"use axum::{{extract::FromRequestParts, http::request::Parts}};

use crate::infra::error::AppError;

/// Custom extractor for {CamelCaseName}.
///
/// Usage in handler:
/// ```rust
/// pub async fn my_handler({CamelCaseName}(val): {CamelCaseName}) -> Result<..., AppError> {{ ... }}
/// ```
pub struct {CamelCaseName}(pub String);

impl<S> FromRequestParts<S> for {CamelCaseName}
where
    S: Send + Sync + 'static,
{{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {{
        // TODO: Implement extractor validation/retrieval logic
        // Example: check authorization headers, custom cookie extraction, etc.
        Ok({CamelCaseName}("extracted_value".to_string()))
    }}
}}
"#,
        CamelCaseName = camel_case
    );

    if let Err(e) = write_file(target_file, &extractor_content) {
        eprintln!("Failed to write extractor file: {}", e);
        std::process::exit(1);
    }

    register_extractor_in_mod(name, &camel_case);

    println!("Extractor '{}' generated successfully!", name);
}
