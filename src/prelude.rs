//! Auto-import of all preroll extension traits.

#[cfg(feature = "postgres")]
pub use crate::middleware::postgres::PostgresRequestExt;
