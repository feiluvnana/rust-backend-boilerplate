use std::fs;
use std::path::Path;

pub fn register_routes_in_mod(feature_name: &str, kebab_case: &str) {
    let mod_file_path = Path::new("src/routes/mod.rs");
    if !mod_file_path.exists() {
        return;
    }

    if let Ok(mut content) = fs::read_to_string(mod_file_path) {
        let mod_decl = format!("pub mod {};", feature_name);
        if !content.contains(&mod_decl) {
            if let Some(pos) = content.find("pub mod health;") {
                content.insert_str(pos, &format!("pub mod {};\n", feature_name));
            } else {
                content.push_str(&format!("\npub mod {};\n", feature_name));
            }
        }

        let nest_decl = format!(".nest(\"/{}\", {}::router())", kebab_case, feature_name);
        if !content.contains(&nest_decl) {
            if let Some(pos) = content.find(".nest(\"/health\"") {
                content.insert_str(pos, &format!(".nest(\"/{}\", {}::router())\n        ", kebab_case, feature_name));
            }
        }

        if let Err(e) = fs::write(mod_file_path, content) {
            eprintln!("Failed to auto-register routes in mod.rs: {}", e);
        }
    }
}
