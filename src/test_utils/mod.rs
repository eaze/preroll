//! Utilities for setting up mock clients and test servers with similar features to `preroll::main!`.
//!
//! See [**preroll-example** on GitHub](https://github.com/eaze/preroll/tree/latest/preroll-example) for a full example of how to integrate test_utils for a service.
//!
//! ## Example:
//!
//! ```
//! use preroll::test_utils::{self, assert_status, TestResult};
//!
//! # #[allow(unused_mut)]
//! pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
//!   // Normally imported from your service's crate (lib.rs).
//! }
//!
//! #[async_std::main] // Would be #[async_std::test] instead.
//! async fn main() -> TestResult<()> {
//!     let client = test_utils::create_client((), setup_routes).await.unwrap();
//!
//!     let mut res = client.get("/monitor/ping").await.unwrap();
//!
//!     let body = assert_status(&mut res, 200).await;
//!     assert_eq!(body, "preroll_test_utils");
//!     Ok(())
//! }
//! ```

#![allow(clippy::unwrap_used)]

use std::convert::TryInto;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

use cfg_if::cfg_if;
use surf::{Client, Config, StatusCode, Url};
use tide::{http, Server};

use crate::builtins::monitor::setup_monitor;
use crate::logging::{log_format_json, log_format_pretty};
use crate::middleware::json_error::JsonError;
use crate::middleware::{JsonErrorMiddleware, LogMiddleware, RequestIdMiddleware};
use crate::VariadicRoutes;

#[cfg(feature = "honeycomb")]
use tracing_subscriber::Registry;

cfg_if! {
    if #[cfg(feature = "postgres")] {
        use async_std::sync::RwLock;
        use sqlx::postgres::{PgConnectOptions, PgPoolOptions, Postgres};
        use sqlx::ConnectOptions;
        use tide::{Middleware, Next, Request};

        use crate::middleware::postgres::{ConnectionWrap, ConnectionWrapInner};
    }
}

/// The result type to use for tests.
///
/// This is a `surf::Result<T>`.
pub type TestResult<T> = surf::Result<T>;

/// Creates a test application with routes and mocks set up,
/// and hands back a client which is already connected to the server.
///
/// ## Example:
///
/// ```
/// use preroll::test_utils::{self, assert_status, TestResult};
///
/// # #[allow(unused_mut)]
/// pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
///   // Normally imported from your service's crate (lib.rs).
/// }
///
/// #[async_std::main] // Would be #[async_std::test] instead.
/// async fn main() -> TestResult<()> {
///     let client = test_utils::create_client((), setup_routes).await.unwrap();
///
///     let mut res = client.get("/monitor/ping").await.unwrap();
///
///     let body = assert_status(&mut res, 200).await;
///     assert_eq!(body, "preroll_test_utils");
///     Ok(())
/// }
/// ```
pub async fn create_client<State>(
    state: State,
    setup_routes_fns: impl Into<VariadicRoutes<State>>,
) -> TestResult<Client>
where
    State: Send + Sync + 'static,
{
    let server = create_server(state, setup_routes_fns)?;

    let client: Client = Config::new()
        .set_http_client(server)
        .set_base_url(Url::parse("http://localhost:8080")?) // Address not actually used.
        .try_into()?;

    Ok(client)
}

/// Creates a test application with routes and mocks set up,
/// and hands back a client which is already connected to the server.
///
/// This function also hands back a postgres transaction connection which is
/// being used for the rest of the application, allowing easy rollback of everything.
///
/// ## Important!
///
/// The `RwLockWriteGuard` returned from `pg_conn.write().await` MUST be [dropped][] before running
/// the test cases, or else there will be a writer conflict and the test will hang indefinitely.
///
/// ## Example:
///
/// ```no_run
/// use preroll::test_utils::{self, TestResult};
///
/// # #[allow(unused_mut)]
/// pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
///   // Normally imported from your service's crate (lib.rs).
/// }
///
/// #[async_std::main] // Would be #[async_std::test] instead.
/// async fn main() -> TestResult<()> {
///     let (client, pg_conn) = test_utils::create_client_and_postgres((), setup_routes).await.unwrap();
///
///     {
/// #       #[allow(unused_mut)]
///         let mut pg_conn = pg_conn.write().await;
///
///         // ... (test setup) ...
///
///         // The RwLockWriteGuard here MUST be dropped before running the test cases,
///         // or else there is a writer conflict and the test hangs indefinitely.
///         //
///         // Note: this is done automatically at the end of the closure.
///         // We are still explicitly dropping so as to avoid accidently messing this up in the future.
///         std::mem::drop(pg_conn);
///     }
///
///     // ... (test cases) ...
///
///     Ok(())
/// }
/// ```
///
/// [dropped]: https://doc.rust-lang.org/reference/destructors.html
#[cfg(feature = "postgres")]
#[cfg_attr(feature = "docs", doc(cfg(feature = "postgres")))]
pub async fn create_client_and_postgres<State>(
    state: State,
    setup_routes_fns: impl Into<VariadicRoutes<State>>,
) -> TestResult<(Client, Arc<RwLock<ConnectionWrapInner<Postgres>>>)>
where
    State: Send + Sync + 'static,
{
    let mut server = create_server(state, setup_routes_fns)?;

    // Fake PostgresConnectionMiddleware.
    //
    // We do this so that all connections within any test run can share the same Transaction and be rolled back on Drop.
    let mut connect_opts = PgConnectOptions::new()
        .host("localhost")
        .database("database_test");
    connect_opts.log_statements(log::LevelFilter::Debug);

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await?;

    let conn_wrap = Arc::new(RwLock::new(ConnectionWrapInner::Transacting(
        pg_pool.begin().await?,
    )));
    server.with(PostgresTestMiddleware(conn_wrap.clone()));

    let mut client = Client::with_http_client(server);
    client.set_base_url(Url::parse("http://localhost:8080")?); // Address not actually used.

    Ok((client, conn_wrap))
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn create_server<State>(
    state: State,
    setup_routes_fns: impl Into<VariadicRoutes<State>>,
) -> TestResult<Server<Arc<State>>>
where
    State: Send + Sync + 'static,
{
    dotenv::dotenv().ok();

    let log_level: log::LevelFilter = env::var("LOGLEVEL")
        .map(|v| v.parse().expect("LOGLEVEL must be a valid log level."))
        .unwrap_or(log::LevelFilter::Off);

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    if environment.starts_with("prod") {
        // Like Production
        env_logger::builder()
            .format(log_format_json)
            .filter_level(log_level)
            .write_style(env_logger::WriteStyle::Never)
            .try_init()
            .ok();
    } else {
        // Like Development
        env_logger::builder()
            .format(log_format_pretty)
            .filter_level(log_level)
            .try_init()
            .ok();
    }

    #[cfg(feature = "honeycomb")]
    {
        let subscriber = Registry::default();
        // .with(tracing_subscriber::fmt::Layer::default()) // log to stdout
        tracing::subscriber::set_global_default(subscriber).ok();
    }

    let mut server = tide::with_state(Arc::new(state));
    server.with(RequestIdMiddleware::new());
    server.with(LogMiddleware::new());
    server.with(JsonErrorMiddleware::new());

    setup_monitor("preroll_test_utils", &mut server);

    let mut version = 1;
    for routes_fn in setup_routes_fns.into().routes {
        routes_fn(server.at(&format!("/api/v{}", version)));
        version += 1;
    }

    Ok(server)
}

#[cfg(feature = "postgres")]
#[cfg_attr(feature = "docs", doc(cfg(feature = "postgres")))]
#[derive(Debug, Clone)]
struct PostgresTestMiddleware(ConnectionWrap<Postgres>);

#[cfg(feature = "postgres")]
#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for PostgresTestMiddleware {
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> tide::Result {
        req.set_ext(self.0.clone());
        Ok(next.run(req).await)
    }
}

/// Creates a mock client directly connected to a server which is setup by the provided function.
///
/// ## Example:
/// ```
/// use preroll::test_utils;
/// use tide::Server;
///
/// fn setup_example_local_org_mocks(mock: &mut Server<()>) {
///     mock.at("hello-world").get(|_| async { Ok("Hello World!") });
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let client = test_utils::mock_client("http://api.example_local.org/", setup_example_local_org_mocks);
///
///     let response = client
///         .get("http://api.example_local.org/hello-world")
///         .recv_string()
///         .await
///         .unwrap();
///
///     assert_eq!(response, "Hello World!");
/// }
/// ```
pub fn mock_client<MocksFn>(base_url: impl AsRef<str>, setup_mocks_fn: MocksFn) -> Client
where
    MocksFn: Fn(&mut Server<()>),
{
    let mut mocks_server = tide::new();
    setup_mocks_fn(&mut mocks_server);

    let mock_client: Client = Config::new()
        .set_http_client(mocks_server)
        .set_base_url(Url::parse(base_url.as_ref()).unwrap())
        .try_into()
        .expect("async-h1 client from config is infallible");

    mock_client
}

/// A test helper to check all fields of a [`JsonError`][crate::JsonError].
///
/// ## Example:
///
/// ```
/// use preroll::test_utils::{self, assert_json_error, TestResult};
///
/// # #[allow(unused_mut)]
/// pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
///     // Normally imported from your service's crate (lib.rs).
/// }
///
/// #[async_std::main] // Would be #[async_std::test] instead.
/// async fn main() -> TestResult<()> {
///     let client = test_utils::create_client((), setup_routes).await.unwrap();
///
///     let mut res = client.get("/not_found").await.unwrap();
///
///     assert_json_error(
///         &mut res,
///         404,
///         "(no additional context)",
///     )
///     .await;
///
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Not actually dead code. (??)
#[track_caller]
pub async fn assert_json_error<Status>(
    mut res: impl AsMut<http::Response>,
    status: Status,
    err_msg: &str,
) where
    Status: TryInto<StatusCode>,
    Status::Error: Debug,
{
    let res = res.as_mut();

    let status: StatusCode = status
        .try_into()
        .expect("test must specify valid status code");

    let str_response = res.body_string().await.unwrap();

    let error: JsonError = serde_json::from_str(&str_response).map_err(|e| {
        surf::Error::from_str(
            res.status(),
            format!("Error, could not parse Response into JsonError! json err: \"{}\", response body: \"{}\"", e, str_response)
        )
    }).unwrap();

    assert_eq!(res.status(), status);
    assert_eq!(&error.title, status.canonical_reason());
    assert_eq!(error.message, err_msg);
    assert_eq!(error.status, status as u16);
    assert_eq!(
        error.request_id.as_str(),
        res["X-Request-Id"].last().as_str()
    );
    if res.status().is_server_error() {
        assert_eq!(
            error
                .correlation_id
                .expect("Internal server errors must have correlation ids.")
                .as_str(),
            res["X-Correlation-Id"].last().as_str()
        );
    } else {
        assert_eq!(error.correlation_id, None);
        assert!(res.header("X-Correlation-Id").is_none());
    }
}

/// Assert that a response has a status code and parse out the body to JSON if possible.
///
/// This helper has better assertion failure messages than doing this manually.
///
/// ## Example:
///
/// ```
/// use preroll::test_utils::{self, assert_status_json, TestResult};
/// use preroll::JsonError;
///
/// # #[allow(unused_mut)]
/// pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
///   // Normally imported from your service's crate (lib.rs).
/// }
///
/// #[async_std::main] // Would be #[async_std::test] instead.
/// async fn main() -> TestResult<()> {
///     let client = test_utils::create_client((), setup_routes).await.unwrap();
///
///     let mut res = client.get("/not_found").await.unwrap();
///
///     let json: JsonError = assert_status_json(&mut res, 404).await;
///     assert_eq!(&json.title, res.status().canonical_reason());
///
///     Ok(())
/// }
/// ```
#[track_caller]
pub async fn assert_status_json<StructType, Status>(
    mut res: impl AsMut<http::Response>,
    status: Status,
) -> StructType
where
    StructType: serde::de::DeserializeOwned,
    Status: TryInto<StatusCode>,
    Status::Error: Debug,
{
    let res = res.as_mut();

    let status: StatusCode = status
        .try_into()
        .expect("test must specify valid status code");

    let body = res.body_string().await.unwrap();

    assert_eq!(res.status(), status, "Response body: {}", body);

    serde_json::from_str(&body).unwrap_or_else(|err| {
        panic!(
            "Error: \"{}\" Body was not parseable into a {}, body was: \"{}\"",
            err,
            std::any::type_name::<StructType>(),
            body
        )
    })
}

/// Assert that a response has a specified status code and return the body as a string.
///
/// This helper has better assertion failure messages than doing this manually.
///
/// ## Example:
///
/// ```
/// use preroll::test_utils::{self, assert_status, TestResult};
///
/// # #[allow(unused_mut)]
/// pub fn setup_routes(mut server: tide::Route<'_, std::sync::Arc<()>>) {
///   // Normally imported from your service's crate (lib.rs).
/// }
///
/// #[async_std::main] // Would be #[async_std::test] instead.
/// async fn main() -> TestResult<()> {
///     let client = test_utils::create_client((), setup_routes).await.unwrap();
///
///     let mut res = client.get("/monitor/ping").await.unwrap();
///
///     let body = assert_status(&mut res, 200).await;
///     assert_eq!(body, "preroll_test_utils");
///     Ok(())
/// }
/// ```
#[track_caller]
pub async fn assert_status<Status>(mut res: impl AsMut<http::Response>, status: Status) -> String
where
    Status: TryInto<StatusCode>,
    Status::Error: Debug,
{
    let res = res.as_mut();

    let status: StatusCode = status
        .try_into()
        .expect("test must specify valid status code");

    let body = res.body_string().await.unwrap();

    assert_eq!(res.status(), status, "Response body: {}", body);

    body
}
