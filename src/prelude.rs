//! Auto-import of all preroll extension traits.

#[cfg(feature = "postgres")]
#[cfg_attr(feature = "docs", doc(cfg(feature = "postgres")))]
pub use crate::middleware::postgres::PostgresRequestExt;
