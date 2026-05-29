use std::env;

mod generators;
mod registrars;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    // Support commands:
    // make g:feature name=xxx -> g feature xxx
    // make g:resource name=xxx -> g resource xxx
    // make g:middleware name=xxx -> g middleware xxx
    // make g:extractor name=xxx -> g extractor xxx
    // default (e.g. g xxx) -> g resource xxx
    let (command, feature_name) = if args.len() == 2 {
        ("resource", &args[1])
    } else {
        (args[1].as_str(), &args[2])
    };

    if !utils::is_valid_snake_case(feature_name) {
        eprintln!("Error: Name must be in snake_case format (e.g., 'user_profile').");
        std::process::exit(1);
    }

    match command {
        "feature" => generators::feature::generate(feature_name),
        "resource" => generators::resource::generate(feature_name),
        "middleware" => generators::middleware::generate(feature_name),
        "extractor" => generators::extractor::generate(feature_name),
        _ => {
            eprintln!("Error: Unknown command '{}'.", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  make g:feature name=<name>     (Creates a basic feature starter)");
    eprintln!("  make g:resource name=<name>    (Creates a NestJS-like CRUD resource)");
    eprintln!("  make g:middleware name=<name>  (Creates a custom Axum middleware)");
    eprintln!("  make g:extractor name=<name>   (Creates a custom Axum extractor)");
}
