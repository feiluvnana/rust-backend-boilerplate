use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn register_middleware_in_mod(name: &str) {
    let mod_file_path = Path::new("src/middleware/mod.rs");
    if mod_file_path.exists() {
        let register_line = format!("pub mod {};", name);
        if let Ok(content) = fs::read_to_string(mod_file_path) {
            if !content.contains(&register_line) {
                println!("Registering middleware in {}...", mod_file_path.display());
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(mod_file_path)
                    .expect("Failed to open middleware mod.rs for appending");
                if let Err(e) = writeln!(file, "pub mod {};", name) {
                    eprintln!("Failed to append middleware registration: {}", e);
                }
            }
        }
    }
}
