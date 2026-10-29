//! Plugin architecture and built-in query transformers.

pub mod camel_case;
pub mod deduplicate_joins;
pub mod traits;
pub mod with_schema;

pub use camel_case::{to_camel_case, to_snake_case, CamelCaseOptions, CamelCasePlugin};
pub use deduplicate_joins::DeduplicateJoinsPlugin;
pub use traits::KyselyPlugin;
pub use with_schema::WithSchemaPlugin;
