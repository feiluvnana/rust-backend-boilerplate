use std::path::Path;

use crate::registrars::middleware::register_middleware_in_mod;
use crate::utils::write_file;

pub fn generate(name: &str) {
    let target_file_str = format!("src/middleware/{}.rs", name);
    let target_file = Path::new(&target_file_str);

    if target_file.exists() {
        eprintln!(
            "Error: Middleware file '{}' already exists.",
            target_file.display()
        );
        std::process::exit(1);
    }

    let middleware_content = format!(
        r#"use axum::{{extract::Request, http::StatusCode, middleware::Next, response::Response}};

/// {CamelName} middleware.
///
/// Add to your router with:
/// ```rust
/// .layer(axum::middleware::from_fn({SnakeName}_middleware))
/// ```
pub async fn {SnakeName}_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {{
    // TODO: Implement middleware logic here
    
    let response = next.run(request).await;
    Ok(response)
}}
"#,
        CamelName = crate::utils::to_camel_case(name),
        SnakeName = name
    );

    if let Err(e) = write_file(target_file, &middleware_content) {
        eprintln!("Failed to write middleware file: {}", e);
        std::process::exit(1);
    }

    register_middleware_in_mod(name);

    println!("Middleware '{}' generated successfully!", name);
}
