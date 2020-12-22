use cfg_if::cfg_if;

pub mod extension_types;
pub mod json_error;
pub mod logger;
pub mod requestid;

pub use json_error::JsonErrorMiddleware;
pub use logger::LogMiddleware;
pub use requestid::RequestIdMiddleware;

cfg_if! {
    if #[cfg(feature = "honeycomb")] {
        #[doc(hidden)]
        pub mod honeycomb;

        #[cfg_attr(feature = "docs", doc(cfg(feature = "honeycomb")))]
        pub mod trace;

        #[cfg_attr(feature = "docs", doc(cfg(feature = "honeycomb")))]
        pub use trace::TraceMiddleware;
    }
}

cfg_if! {
    if #[cfg(feature = "postgres")] {
        #[cfg_attr(feature = "docs", doc(cfg(feature = "postgres")))]
        pub mod postgres;

        #[cfg_attr(feature = "docs", doc(cfg(feature = "postgres")))]
        pub use postgres::{PostgresMiddleware, PostgresRequestExt};
    }
}
