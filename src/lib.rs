//! Easy boilerplate utilities for Rust http services which use [async-std][], [Tide][], [Surf][], and friends.
//!
//! This crate is intentionally somewhat prescrptive in how it templates a service and the interaction with
//! add-on features such as Postgres (via [SQLx][]).
//!
//! **Scroll to the bottom for API Reference**
//!
//! ## Features
//!
//! - Boilerplate `main` setup via `preroll::main!`, with optional features automatically configured.
//! - A `preroll::prelude::*;` with all extension traits.
//! - Response logging with many details.
//! - Automatic JSON reponses for errors.
//! - Test utils with easy mock client setup.
//!
//! ## Optional features
//! Add-on features must be enabled via cargo features, e.g.
//!
//! ```toml
//! [dependencies.preroll]
//! version = "0.1"
//! features = ["honeycomb", "postgres"]
//! ```
//!
//! ### List of optional add-on features:
//! - `"honeycomb"`: Enables tracing to [honeycomb.io].
//!     - Env variable `HONEYCOMBIO_WRITE_KEY` (required).
//!     - Env variable `TRACELEVEL`, sets the tracing level filter, defaults to `info`.
//!     - Writes to a dataset named `{service_name}-{environment}`.
//!         - `service_name` is from `preroll::main!("service_name", ...)`.
//!         - `environment` is from `ENVIRONMENT`, or defaults to `"development"`.
//! - `"postgres"`: Enables a postgres connection pool with transactions.
//!     - Env variable `PGURL`, which should be a properly formatted `postgres://` database url.
//!         - Defaults to `"postgres://localhost/{service_name}"` (default postgres port).
//!         - `service_name` is from `preroll::main!("service_name", ...)`.
//!     - Env variable `PGMAXCONNECTIONS`, default 5 connections.
//!     - Enables [`PostgresRequestExt`][prelude::PostgresRequestExt] and [`test_utils::create_client_and_postgres`][].
//!
//! ### List of other optional features:
//! - `"panic-on-error"`: Makes the response logger [panic][] on error rather than log.
//!     - Do not use in production. Prevents `--release` compilation.
//!
//! ## General Environment Settings
//! The following environment variables are read during `preroll::main!`:
//! - `ENVIRONMENT`: If this starts with `prod`, load the production-mode JSON logger, avoid `.env`.
//! - `FORCE_DOTENV`: Override production-mode, force-load environment from `.env`.
//! - `HOST`: Sets the hostname that this service will listen on. Defaults to `"127.0.0.1"`.
//! - `LOGLEVEL`: Set the logger's level filter, defaults to `info` in production-mode, `debug` in development-mode.
//! - `PORT`: Sets the port that this service will listen on. Defaults to `8080`.
//!
//! [async-std]: https://async.rs/
//! [honeycomb.io]: https://www.honeycomb.io/
//! [SQLx]: https://github.com/launchbadge/sqlx#sqlx
//! [Surf]: https://github.com/http-rs/surf#surf
//! [Tide]: https://github.com/http-rs/tide#tide

#![forbid(unsafe_code, future_incompatible)]
#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    unused_qualifications
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]
#![deny(
    clippy::debug_assert_with_mut_call,
    clippy::exit,
    // clippy::future_not_send,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    // clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::wildcard_dependencies,
)]
#![warn(
    clippy::dbg_macro,
    clippy::macro_use_imports,
    // clippy::multiple_crate_versions,
    clippy::needless_borrow,
    // clippy::panic, // Interferes with SQLx macros
    clippy::print_stdout,
    clippy::trait_duplication_in_bounds,
    clippy::type_repetition_in_bounds,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    // clippy::used_underscore_binding, // Interferes with SQLx macros
)]
#![cfg_attr(feature = "docs", feature(doc_cfg))]

#[cfg(all(not(debug_assertions), feature = "panic-on-error"))]
compile_error!("The \"panic-on-error\" feature must not be used in production, and is not available with `--release`.");

pub(crate) mod logging;
pub(crate) mod middleware;

#[doc(hidden)]
pub mod setup;

pub mod prelude;
pub mod test_utils;
pub mod utils;

/// The result type which is expected from functions passed to `preroll::main!`.
///
/// This is a `color_eyre::eyre::Result<T>`.
pub type SetupResult<T> = setup::Result<T>;

/// A macro which constructs the equivalent of an `async fn main()`.
///
/// Automatically pulls in setup for preroll's default and optional features.
///
/// This macro takes up to three arguments as follows:
///
/// ## service_name
/// The constant service name, staticly set in the service's code.
///
/// A **`&'static str`**.
///
/// ## (optional) state_setup
/// This is where server state can be set.
///
/// An **`async fn setup_state() -> preroll::setup::Result<AppState>`**, where `AppState` is anything which can be thread-safe.
/// That is the state must implement `Send + Sync`, (usually automatically), and must have a `'static` liftime (must be owned).
///
/// It is expected that `AppState` is some arbitrary custom type. `preroll` will wrap it in an [`Arc`][] so that is can be shared.
///
/// This function must be `async` and must return a `preroll::setup::Result`.
/// It is expected that setup could be anything and may need to await or error.
///
/// See [`tide::Server::with_state()`][] from more on Tide server state.
///
/// ## routes_setup
/// This is where routes must be set.
///
/// A **`fn setup_routes(server: &mut Server<Arc<AppState>>)`** where `AppState` is the type returned from `setup_state` or else the [unit `()`][] type.
///
/// It is expected that only Tide route handlers are set in this function. It must not be async and must not error.
///
/// See [`tide::Server::at()`][] for more on Tide server routing.
///
/// ## Example
///
/// ```no_run
/// use std::sync::Arc;
///
/// use tide::{Request, Server};
///
/// # #[allow(dead_code)]
/// struct AppState {
///     greeting: &'static str,
/// }
///
/// # #[allow(dead_code)]
/// type AppRequest = Request<Arc<AppState>>;
///
/// # #[allow(dead_code)]
/// async fn setup_app_state() -> preroll::setup::Result<AppState> {
///     Ok(AppState {
///         greeting: "Hello World!",
///     })
/// }
///
/// # #[allow(dead_code)]
/// fn setup_routes(server: &mut Server<Arc<AppState>>) {
///     server
///         .at("hello-world")
///         .get(|req: AppRequest| async move {
///             Ok(req.state().greeting)
///         });
/// }
///
/// preroll::main!("hello-world", setup_app_state, setup_routes);
/// ```
///
/// [`tide::Server::at()`]: https://docs.rs/tide/0.15.0/tide/struct.Server.html#method.at
/// [`tide::Server::with_state()`]: https://docs.rs/tide/0.15.0/tide/struct.Server.html#method.with_state
/// [unit `()`]: https://doc.rust-lang.org/std/primitive.unit.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
#[macro_export]
macro_rules! main {
    // preroll::main!("service-name", routes_setup_function);
    ($service_name:tt, $routes_setup:tt) => {
        $crate::main!(service_name, (), async { () }, routes_setup);
    };

    // preroll::main!("service-name", state_setup_function, routes_setup_function);
    ($service_name:tt, $state_setup:tt, $routes_setup:tt) => {
        fn main() -> preroll::setup::Result<()> {
            preroll::setup::block_on(async {
                preroll::setup::initial_setup($service_name)?;

                let state = $state_setup().await?;

                let mut server = preroll::setup::setup_server($service_name, state).await?;

                $routes_setup(&mut server);

                preroll::setup::start_server(server).await?;

                Ok(())
            })
        }
    };
}
