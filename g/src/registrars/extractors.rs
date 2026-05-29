use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn register_extractor_in_mod(name: &str, camel_case: &str) {
    let mod_file_path = Path::new("src/extractors/mod.rs");
    if mod_file_path.exists() {
        let mod_decl = format!("pub mod {};", name);
        let use_decl = format!("pub use {}::{};", name, camel_case);
        if let Ok(content) = fs::read_to_string(mod_file_path) {
            if !content.contains(&mod_decl) || !content.contains(&use_decl) {
                println!("Registering extractor in {}...", mod_file_path.display());
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(mod_file_path)
                    .expect("Failed to open extractors mod.rs for appending");
                if !content.contains(&mod_decl) {
                    if let Err(e) = writeln!(file, "pub mod {};", name) {
                        eprintln!("Failed to append extractor mod decl: {}", e);
                    }
                }
                if !content.contains(&use_decl) {
                    if let Err(e) = writeln!(file, "pub use {}::{};", name, camel_case) {
                        eprintln!("Failed to append extractor use decl: {}", e);
                    }
                }
            }
        }
    }
}
