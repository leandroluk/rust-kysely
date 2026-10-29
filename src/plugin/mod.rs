//! Plugin architecture and built-in query transformers.

pub mod camel_case;
pub mod traits;

pub use camel_case::{to_camel_case, to_snake_case, CamelCaseOptions, CamelCasePlugin};
pub use traits::KyselyPlugin;
