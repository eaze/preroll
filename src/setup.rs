//! The setup components which comprise `preroll::main!`.
//!
//! These are exposed in the event they need to be used more manually, but use is discouraged.
//! Prefer using `preroll::main!` whenever possible.

use std::env;
use std::future::Future;
use std::sync::Arc;

use cfg_if::cfg_if;
use tide::{Request, Server};

pub use async_std::task::block_on;

use crate::builtins::monitor::setup_monitor;

cfg_if! {
    if #[cfg(feature = "honeycomb")] {
        use tracing_honeycomb::new_blackhole_telemetry_layer;
        use tracing_subscriber::filter::LevelFilter;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::Registry;

        use crate::middleware::TraceMiddleware;
    }
}

cfg_if! {
    if #[cfg(feature = "postgres")] {
        use std::time::Duration;

        use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
        use sqlx::ConnectOptions;

        use crate::middleware::PostgresMiddleware;
    }
}

cfg_if! {
    if #[cfg(feature = "lambda-http")] {
        use tide_lambda_listener::LambdaListener;
    } else {
        use tide::listener::Listener;
    }
}

use crate::logging::{log_format_json, log_format_pretty};
use crate::middleware::{
    ClacksMiddleware, JsonErrorMiddleware, LogMiddleware, RequestIdMiddleware,
};
use crate::VariadicRoutes;

/// The result type which is expected from functions passed to `preroll::main!`,
/// and used in the return of `setup`'s functions.
///
/// This is a `color_eyre::eyre::Result<T>`.
pub type Result<T> = color_eyre::eyre::Result<T>;

pub async fn setup<AppState, StateFn, StateFnFuture, ServerFn, ServerFnFuture>(
    service_name: &'static str,
    state_setup: StateFn,
    server_setup: ServerFn,
    routes_setups: impl Into<VariadicRoutes<AppState>>,
) -> Result<()>
where
    AppState: Send + Sync + 'static,
    StateFn: Fn() -> StateFnFuture,
    StateFnFuture: Future<Output = Result<AppState>>,
    ServerFn: Fn(Server<Arc<AppState>>) -> ServerFnFuture,
    ServerFnFuture: Future<Output = Result<Server<Arc<AppState>>>>,
{
    initial_setup(service_name)?;

    let state = state_setup().await?;

    let (mut base_server, server) = setup_server(service_name, state).await?;

    let mut server = server_setup(server).await?;

    let mut version = 1;
    for routes_fn in routes_setups.into().routes {
        routes_fn(server.at(&format!("/api/v{}", version)));
        version += 1;
    }

    #[cfg(debug_assertions)]
    server.at("/internal-error").get(get_internal_error);

    base_server.at("/").nest(server);
    start_server(base_server).await?;

    Ok(())
}

#[cfg(debug_assertions)]
async fn get_internal_error<AppState>(_req: Request<Arc<AppState>>) -> tide::Result<&'static str>
where
    AppState: Send + Sync + 'static,
{
    Err(tide::Error::from_str(
        500,
        "Intentional Server Error from GET /internal-error",
    ))
}

#[cfg_attr(not(feature = "honeycomb"), allow(unused_variables))]
pub fn initial_setup(service_name: &'static str) -> Result<()> {
    color_eyre::install()?;

    let log_level = env::var("LOGLEVEL")
        .map(|v| v.parse().expect("LOGLEVEL must be a valid log level."))
        .unwrap_or(log::LevelFilter::Info);

    if env::var("FORCE_DOTENV").is_ok() || env::var("DEBUG_DOTENV").is_ok() {
        dotenv::dotenv().ok();
    }

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    // Logging
    if environment.starts_with("prod") {
        env_logger::builder()
            .format(log_format_json)
            .filter_level(log_level)
            .write_style(env_logger::WriteStyle::Never)
            .try_init()?;
    } else {
        // Development
        dotenv::dotenv().ok();

        env_logger::builder()
            .format(log_format_pretty)
            .filter_level(log_level)
            .try_init()?;
    }

    log::info!("Logger started - level: {}", log_level);

    // Tracing (Honeycomb)
    #[cfg(feature = "honeycomb")]
    {
        let trace_filter: LevelFilter = env::var("TRACELEVEL")
            .map(|v| v.parse())
            .unwrap_or(Ok(LevelFilter::INFO))?;

        if let Ok(api_key) = env::var("HONEYCOMB_WRITEKEY") {
            let maybe_sample_rate = env::var("HONEYCOMB_SAMPLE_RATE");

            #[cfg(feature = "lambda-http")]
            let telemetry_layer = {
                // Just to avoid unused warnings.
                // In this setup the environemnt's consumer will have to have this.
                drop(api_key);

                let telemetry_builder = tracing_honeycomb::Builder::new_stdout(service_name);

                if let Ok(sample_rate) = maybe_sample_rate.unwrap_or_default().parse() {
                    telemetry_builder.with_trace_sampling(sample_rate /* u32 */)
                } else {
                    telemetry_builder
                }
                .build()
            };

            #[cfg(not(feature = "lambda-http"))]
            let telemetry_layer = {
                let dataset = env::var("HONEYCOMB_DATASET")
                    .unwrap_or_else(|_| format!("{}-{}", service_name, environment));

                let api_host = env::var("HONEYCOMB_API_HOST")
                    .unwrap_or_else(|_| "https://api.honeycomb.io/".to_string());

                let honeycomb_config = libhoney::Config {
                    options: libhoney::client::Options {
                        api_key,
                        api_host,
                        dataset,
                        ..libhoney::client::Options::default()
                    },
                    transmission_options: libhoney::transmission::Options::default(),
                };

                let telemetry_builder =
                    tracing_honeycomb::Builder::new_libhoney(service_name, honeycomb_config);

                if let Ok(sample_rate) = maybe_sample_rate.unwrap_or_default().parse() {
                    telemetry_builder.with_trace_sampling(sample_rate /* u32 */)
                } else {
                    telemetry_builder
                }
                .build()
            };

            let subscriber = Registry::default()
                .with(trace_filter) // filter out low-level debug tracing
                // .with(tracing_subscriber::fmt::Layer::default()) // log to stdout
                .with(telemetry_layer); // publish to honeycomb backend

            tracing::subscriber::set_global_default(subscriber)?;

            log::info!("Honeycomb Tracing enabled - filter: {}", trace_filter);
        } else {
            let telemetry_layer = new_blackhole_telemetry_layer();

            let subscriber = Registry::default()
                .with(trace_filter) // filter out low-level debug tracing
                // .with(tracing_subscriber::fmt::Layer::default()) // log to stdout
                .with(telemetry_layer); // publish to honeycomb backend

            tracing::subscriber::set_global_default(subscriber)?;

            log::info!("Honeycomb Tracing off");
        }
    }

    Ok(())
}

#[cfg_attr(not(feature = "postgres"), allow(unused_variables))]
pub async fn setup_server<State>(
    service_name: &'static str,
    state: State,
) -> Result<(Server<Arc<()>>, Server<Arc<State>>)>
where
    State: Send + Sync + 'static,
{
    let mut base_server = tide::with_state(Arc::new(()));
    base_server.with(ClacksMiddleware::new());

    // Set handlers for /monitor/ping, etc.
    //
    // These are intentionally excluded from logging/tracing middleware.
    setup_monitor(service_name, &mut base_server);

    let mut server = tide::with_state(Arc::new(state));
    server.with(ClacksMiddleware::new());
    server.with(RequestIdMiddleware::new());
    server.with(LogMiddleware::new());
    server.with(JsonErrorMiddleware::new());

    #[cfg(feature = "honeycomb")]
    server.with(TraceMiddleware::new());

    // Postgres
    #[cfg(feature = "postgres")]
    {
        let max_connections: u32 = env::var("PGMAXCONNECTIONS")
            .map(|v| v.parse())
            .unwrap_or(Ok(5))?;
        let max_lifetime: u64 = env::var("PGMAXLIFETIME")
            .map(|v| v.parse())
            .unwrap_or(Ok(30 /* 30 mins */))?;

        let pgurl =
            env::var("PGURL").unwrap_or_else(|_| format!("postgres://localhost/{}", service_name));

        let mut connect_opts: PgConnectOptions = pgurl.parse()?;
        connect_opts.log_statements(log::LevelFilter::Debug);

        let pg_pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .max_lifetime(Duration::from_secs(max_lifetime * 60 /* to seconds */))
            .connect_with(connect_opts)
            .await?;

        server.with(PostgresMiddleware::from(pg_pool));
    }

    Ok((base_server, server))
}

pub async fn start_server<State>(server: Server<Arc<State>>) -> Result<()>
where
    State: Send + Sync + 'static,
{
    #[cfg(feature = "lambda-http")]
    {
        server.listen(LambdaListener::new()).await?;
    }
    #[cfg(not(feature = "lambda-http"))]
    {
        let port: u16 = env::var("PORT").map(|v| v.parse()).unwrap_or(Ok(8080))?;
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let mut listener = server.bind((host.as_str(), port)).await?;
        for info in listener.info().iter() {
            log::info!("Server listening on {}", info);
        }
        listener.accept().await?;
    }

    // Essentially "never".
    Ok(())
}
