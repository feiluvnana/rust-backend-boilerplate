//! Custom Axum extractors for request validation and data extraction.

pub mod validated_json;
pub mod validated_path;
pub mod validated_query;

pub use validated_json::ValidatedJson;
pub use validated_path::ValidatedPath;
pub use validated_query::ValidatedQuery;
