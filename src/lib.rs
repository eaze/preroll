//! Easy boilerplate utilities for Rust http services which use [async-std][], [Tide][], [Surf][], and friends.
//!
//! Allows for service setup with feature-configured built-ins for maximum service consistency with low developer overhead,
//! and for easily integration testing the service without using a live network.
//!
//! **Scroll to the bottom for API Reference**
//!
//! ## Example
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! use tide::{Request, Route};
//!
//! # #[allow(dead_code)]
//! struct AppState {
//!     greeting: &'static str,
//! }
//!
//! # #[allow(dead_code)]
//! type AppRequest = Request<Arc<AppState>>;
//!
//! # #[allow(dead_code)]
//! async fn setup_app_state() -> preroll::SetupResult<AppState> {
//!     Ok(AppState {
//!         greeting: "Hello World!",
//!     })
//! }
//!
//! # #[allow(dead_code)]
//! fn setup_routes(mut server: Route<'_, Arc<AppState>>) {
//!     server
//!         .at("hello-world")
//!         .get(|req: AppRequest| async move {
//!             Ok(req.state().greeting)
//!         });
//! }
//!
//! // The "magic" happens here!
//! preroll::main!("hello-world", setup_app_state, setup_routes);
//! ```
//!
//! ## Features
//!
//! - Boilerplate `main` setup via [`preroll::main!`][], with optional features automatically configured.
//! - A [`preroll::prelude::*;`][] with all extension traits.
//! - Response logging with many details.
//! - Automatic JSON responses for errors in the form of [`JsonError`][].
//! - [Test utils][] with easy mock client setup.
//!
//! ## Optional features
//! Add-on features must be enabled via cargo features, e.g.
//!
//! ```toml
//! [dependencies.preroll]
//! version = "0.5"
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
//! - `"lambda-http"`: Changes the HTTP listener to connect to an AWS Lambda execution environment.
//!     - Is no longer reachable as a regular http server, but accepts http lambda requests as if it were one.
//!     - Some environment variables, such as `PORT`, are disregarded.
//!     - If the `"honeycomb"` feature is enabled, trace events are written to stdout, and must be collected via
//!         a layer provided by Honeycomb. See: https://docs.honeycomb.io/getting-data-in/integrations/aws/aws-lambda/
//! - `"postgres"`: Enables a postgres connection pool with transactions.
//!     - Env variable `PGURL`, which should be a properly formatted `postgres://` database url.
//!         - Defaults to `"postgres://localhost/{service_name}"` (default postgres port).
//!         - `service_name` is from `preroll::main!("service_name", ...)`.
//!     - Env variable `PGMAXCONNECTIONS`, default 5 connections.
//!     - Env variable `PGMAXLIFETIME`, default `30` (minutes).
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
//! ## Note:
//!
//! This crate is intentionally somewhat prescriptive in how it templates a service and the interaction with
//! add-on features such as Postgres (via [SQLx][]).
//!
//! [`preroll::main!`]: https://docs.rs/preroll/0.8.0/preroll/macro.main.html
//! [`preroll::prelude::*;`]: https://docs.rs/preroll/0.8.0/preroll/prelude/index.html
//! [`JsonError`]: https://docs.rs/preroll/0.8.0/preroll/struct.JsonError.html
//! [async-std]: https://async.rs/
//! [honeycomb.io]: https://www.honeycomb.io/
//! [SQLx]: https://github.com/launchbadge/sqlx#sqlx
//! [Surf]: https://github.com/http-rs/surf#surf
//! [Test utils]: https://docs.rs/preroll/0.8.0/preroll/test_utils/index.html
//! [Tide]: https://github.com/http-rs/tide#tide

#![forbid(unsafe_code)]
#![deny(future_incompatible)]
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

mod routes_variadic;

pub(crate) mod builtins;
pub(crate) mod logging;
pub(crate) mod middleware;

#[doc(hidden)]
pub mod setup;

pub mod prelude;
pub mod test_utils;
pub mod utils;

/// The format of error responses from preroll's error handling middleware.
pub use middleware::json_error::JsonError;

pub use routes_variadic::VariadicRoutes;

/// The result type which is expected from functions passed to `preroll::main!`.
///
/// This is a `color_eyre::eyre::Result<T>`.
pub type SetupResult<T> = setup::Result<T>;

/// **Begin here.** A macro which constructs the equivalent of an `async fn main() {}`.
///
/// Automatically pulls in setup for preroll's default and optional features.
///
/// This macro takes the following arguments:
///
/// ## `service_name`
/// The constant service name, staticly set in the service's code.
///
/// An [**`&'static str`**](https://doc.rust-lang.org/std/primitive.str.html), e.g. `"service-name"`.
///
/// ## `state_setup` (optional)
/// This is where server state can be set.
///
/// An **`async fn setup_state() -> preroll::SetupResult<State>`**, where `State` is anything which can be thread-safe.
/// That is, the state must implement `Send + Sync`, (usually automatically), and must have the `'static` lifetime (must be [owned][]).
///
/// It is expected that `State` is some arbitrary custom type used by your service. `preroll` will wrap it in an [`Arc`][] so that it can be shared.
///
/// This function must be `async` and must return a `preroll::SetupResult`.
/// It is expected that setup could be anything and may need to await or error.
///
/// See [`tide::Server::with_state()`][] for more on Tide server state.
///
/// ## `custom_setup` (optional) (advanced)
/// Advanced, custom setup with access to the full server struct. Prefer using `routes_setup` whenever possible.
///
/// An **`async fn custom_setup(server: Server<Arc<State>>) -> SetupResult<Server<Arc<State>>>`**, where `State` is the type returned from `setup_state` or else the [unit `()`][] type.
///
/// ## `routes_setup` (one or more)
/// This is where routes should be set.
///
/// A **`fn setup_routes(server: &mut tide::Server<Arc<State>>)`**, where `State` is the type returned from `setup_state` or else the [unit `()`][] type.
/// It is expected that only Tide route handlers are set in this function. It must not be async and must not error.
///
/// ### API Versioning
///
/// Any number of `routes_setup` functions can be provided, by use of [`VariadicRoutes`][crate::VariadicRoutes], which will be API versioned as described below.
/// Usually this is done by putting the routes in a [Tuple][], similar to just adding more arguments but wrapping the routes arguments in parenthesis: `(routes_v1, routes_v2)`.
///
/// Preroll route setup functions are automatically namespaced under `/api/v{N}` where the `{N}` is the position of the routes setup function in
/// `preroll::main!`'s arguments, starting at `1`.
///
/// For example, `preroll::main!("my-service", my_routes)` will have `my_routes` mounted at `/api/v1`.
///
/// See [`tide::Server::at()`][] for more on Tide server routing.
///
/// # Basic Example
///
/// This will respond with `"Hello World!"` when a GET request is made to `$HOST:$PORT/api/v1/hello-world`.
///
/// ```no_run
/// # #[cfg(not(feature = "custom_middleware"))]
/// # {
/// use std::sync::Arc;
///
/// use tide::{Request, Route};
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
/// async fn setup_app_state() -> preroll::SetupResult<AppState> {
///     Ok(AppState {
///         greeting: "Hello World!",
///     })
/// }
///
/// # #[allow(dead_code)]
/// fn setup_routes(mut server: Route<'_, Arc<AppState>>) {
///     server
///         .at("hello-world")
///         .get(|req: AppRequest| async move {
///             Ok(req.state().greeting)
///         });
/// }
///
/// preroll::main!("hello-world", setup_app_state, setup_routes);
/// # }
/// ```
///
/// # Full Example
/// With custom middleware and multiple api versions.
///
/// ```no_run
/// # #[cfg(not(feature = "custom_middleware"))]
/// # {
/// use std::sync::Arc;
///
/// use preroll::SetupResult;
/// use tide::{Request, Route, Server};
///
/// # #[allow(dead_code)]
/// pub struct AppState {
///     greeting: &'static str,
/// }
///
/// # #[allow(dead_code)]
/// type AppRequest = Request<Arc<AppState>>;
///
/// # #[allow(dead_code)]
/// async fn setup_app_state() -> preroll::SetupResult<AppState> {
///     Ok(AppState {
///         greeting: "Hello World!",
///     })
/// }
///
/// # #[allow(dead_code)]
/// pub async fn setup_custom(
///    server: Server<Arc<AppState>>
/// ) -> SetupResult<Server<Arc<AppState>>> {
///    // Adjust `server` in whichever ways neccessary
///    Ok(server)
/// }
///
/// # #[allow(dead_code)]
/// fn setup_routes_v1(mut server: Route<'_, Arc<AppState>>) {
///     server
///         .at("hello-world")
///         .get(|req: AppRequest| async move {
///             Ok(req.state().greeting)
///         });
/// }
///
/// # #[allow(dead_code)]
/// fn setup_routes_v2(mut server: Route<'_, Arc<AppState>>) {
///     server
///         .at("hello-world")
///         .get(|req: AppRequest| async move {
///             Ok("Hello from v2!")
///         });
/// }
///
/// preroll::main!(
///     "hello-world",
///     setup_app_state,
///     setup_custom,
///     (setup_routes_v1, setup_routes_v2)
/// );
/// # }
/// ```
///
/// [`tide::Server::at()`]: https://docs.rs/tide/0.15.0/tide/struct.Server.html#method.at
/// [`tide::Server::with_state()`]: https://docs.rs/tide/0.15.0/tide/struct.Server.html#method.with_state
/// [unit `()`]: https://doc.rust-lang.org/std/primitive.unit.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
/// [owned]: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
/// [Tuple]: https://doc.rust-lang.org/std/primitive.tuple.html
#[macro_export]
macro_rules! main {
    // preroll::main!("service-name", routes_setup_function);
    ($service_name:tt, $routes_fns:tt) => {
        $crate::main!($service_name, async { Ok(()) }, routes_fns);
    };

    // preroll::main!("service-name", state_setup_function, routes_setup_function);
    ($service_name:tt, $state_setup:tt, $routes_fns:tt) => {
        async fn setup_noop<State>(
            server: tide::Server<std::sync::Arc<State>>,
        ) -> preroll::SetupResult<tide::Server<std::sync::Arc<State>>>
        where
            State: Send + Sync + 'static,
        {
            Ok(server)
        }

        $crate::main!($service_name, $state_setup, setup_noop, $routes_fns);
    };

    // preroll::main!("service-name", state_setup_function, custom_setup_function, routes_setup_function(s));
    ($service_name:tt, $state_setup:tt, $custom_setup:tt, $routes_fns:tt) => {
        fn main() -> preroll::setup::Result<()> {
            let fut =
                preroll::setup::setup($service_name, $state_setup, $custom_setup, $routes_fns);

            preroll::setup::block_on(fut)
        }
    };
}
