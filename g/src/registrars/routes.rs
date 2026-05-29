use std::fs;
use std::path::Path;

pub fn register_routes_in_mod(feature_name: &str, kebab_case: &str) {
    let mod_file_path = Path::new("src/routes/mod.rs");
    if !mod_file_path.exists() {
        return;
    }

    if let Ok(mut content) = fs::read_to_string(mod_file_path) {
        let nest_decl = format!(
            ".nest(\"/{}\", crate::features::{}::router::router())",
            kebab_case, feature_name
        );
        if !content.contains(&nest_decl) {
            if let Some(pos) = content.find(".nest(\"/health\"") {
                content.insert_str(
                    pos,
                    &format!(
                        ".nest(\"/{}\", crate::features::{}::router::router())\n        ",
                        kebab_case, feature_name
                    ),
                );
            }
        }

        if let Err(e) = fs::write(mod_file_path, content) {
            eprintln!("Failed to auto-register routes in mod.rs: {}", e);
        }
    }
}
