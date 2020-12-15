use std::env;
use std::sync::Arc;

use cfg_if::cfg_if;
use tide::listener::Listener;
use tide::Server;

cfg_if! {
    if #[cfg(feature = "honeycomb")] {
        use tracing_honeycomb::{new_blackhole_telemetry_layer, new_honeycomb_telemetry_layer};
        use tracing_subscriber::filter::LevelFilter;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::Registry;

        use crate::middleware::TraceMiddleware;
    }
}

cfg_if! {
    if #[cfg(feature = "postgres")] {
        use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
        use sqlx::ConnectOptions;

        use crate::middleware::PostgresMiddleware;
    }
}

pub use async_std::task::block_on;

use crate::middleware::{
    JsonErrorMiddleware, LogMiddleware, RequestIdMiddleware,
};

use crate::utils::{log_format_json, log_format_pretty};

pub type SetupResult<T> = color_eyre::eyre::Result<T>;

#[cfg_attr(not(feature = "honeycomb"), allow(unused_variables))]
pub fn initial_setup(service_name: &'static str) -> SetupResult<()> {
    color_eyre::install()?;

    let log_level: log::LevelFilter;

    if env::var("DEBUG_DOTENV").is_ok() {
        dotenv::dotenv().ok();
    }

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    // Logging
    if environment.starts_with("prod") {
        // Production
        log_level = env::var("LOGLEVEL")
            .map(|v| v.parse().expect("LOGLEVEL must be a valid log level."))
            .unwrap_or(log::LevelFilter::Info);

        env_logger::builder()
            .format(log_format_json)
            .filter_level(log_level)
            .write_style(env_logger::WriteStyle::Never)
            .try_init()?;
    } else {
        // Development
        dotenv::dotenv().ok();

        log_level = env::var("LOGLEVEL")
            .map(|v| v.parse().expect("LOGLEVEL must be a valid log level."))
            .unwrap_or(log::LevelFilter::Debug);

        env_logger::builder()
            .format(log_format_pretty)
            .filter_level(log_level)
            .try_init()?;
    }

    log::info!("Logger started - level: {}", log_level);

    // Tracing (Honeycomb)
    cfg_if! {
        if #[cfg(feature = "honeycomb")] {
            let trace_filter: LevelFilter = env::var("TRACELEVEL")
                .map(|v| v.parse())
                .unwrap_or(Ok(LevelFilter::INFO))?;

            if let Ok(honeycomb_key) = env::var("HONEYCOMBIO_WRITE_KEY") {
                let honeycomb_config = libhoney::Config {
                    options: libhoney::client::Options {
                        api_key: honeycomb_key,
                        dataset: format!("{}-{}", service_name, environment),
                        ..libhoney::client::Options::default()
                    },
                    transmission_options: libhoney::transmission::Options::default(),
                };

                let telemetry_layer = new_honeycomb_telemetry_layer(service_name, honeycomb_config);
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
    }

    Ok(())
}

#[cfg_attr(not(feature = "postgres"), allow(unused_variables))]
pub async fn setup_middleware<State>(
    service_name: &'static str,
    state: State,
) -> SetupResult<Server<Arc<State>>>
where
    State: Send + Sync + 'static,
{
    let mut server = tide::with_state(Arc::new(state));
    server.with(RequestIdMiddleware::new());

    #[cfg(feature = "honeycomb")]
    server.with(TraceMiddleware::new());

    server.with(LogMiddleware::new());
    server.with(JsonErrorMiddleware::new());

    // Postgres
    cfg_if! {
        if #[cfg(feature = "postgres")] {
            let max_connections: u32 = env::var("PGMAXCONNECTIONS")
                .map(|v| v.parse())
                .unwrap_or(Ok(5))?;

            let pgurl =
                env::var("PGURL").unwrap_or_else(|_| format!("postgres://localhost/{}", service_name));

            let mut connect_opts: PgConnectOptions = pgurl.parse()?;
            connect_opts.log_statements(log::LevelFilter::Debug);

            let pg_pool = PgPoolOptions::new()
                .max_connections(max_connections)
                .connect_with(connect_opts)
                .await?;

            server.with(PostgresMiddleware::from(pg_pool));
        }
    }

    Ok(server)
}

pub async fn start_server<State>(server: Server<Arc<State>>) -> SetupResult<()>
where
    State: Send + Sync + 'static,
{
    let port: u16 = env::var("PORT").map(|v| v.parse()).unwrap_or(Ok(8080))?;
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let mut listener = server.bind((host.as_str(), port)).await?;
    for info in listener.info().iter() {
        log::info!("Server listening on {}", info);
    }
    listener.accept().await?;

    Ok(())
}
