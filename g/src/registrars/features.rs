use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn register_feature_in_mod(feature_name: &str) {
    let mod_file_path = Path::new("src/features/mod.rs");
    if mod_file_path.exists() {
        let register_line = format!("pub mod {};", feature_name);
        if let Ok(content) = fs::read_to_string(mod_file_path) {
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
    }
}
