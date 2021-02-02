[![preroll on crates.io](https://img.shields.io/crates/v/preroll)](https://crates.io/crates/preroll) [![Documentation (latest release)](https://docs.rs/preroll/badge.svg)](https://docs.rs/preroll/)

# preroll

Easy boilerplate utilities for Rust http services which use [async-std][], [Tide][], [Surf][], and friends.

Allows for service setup with feature-configured builtins for maximum service consistency with low developer overhead,
and for easily integration testing the service without using a live network.

**Scroll to the bottom for API Reference**

### Example

```rust
use std::sync::Arc;

use tide::{Request, Route};

struct AppState {
    greeting: &'static str,
}

type AppRequest = Request<Arc<AppState>>;

async fn setup_app_state() -> preroll::SetupResult<AppState> {
    Ok(AppState {
        greeting: "Hello World!",
    })
}

fn setup_routes(mut server: Route<'_, Arc<AppState>>) {
    server
        .at("hello-world")
        .get(|req: AppRequest| async move {
            Ok(req.state().greeting)
        });
}

// The "magic" happens here!
preroll::main!("hello-world", setup_app_state, setup_routes);
```

### Features

- Boilerplate `main` setup via [`preroll::main!`][], with optional features automatically configured.
- A [`preroll::prelude::*;`][] with all extension traits.
- Response logging with many details.
- Automatic JSON reponses for errors in the form of [`JsonError`][].
- [Test utils][] with easy mock client setup.

### Optional features
Add-on features must be enabled via cargo features, e.g.

```toml
[dependencies.preroll]
version = "0.2"
features = ["honeycomb", "postgres"]
```

#### List of optional add-on features:
- `"honeycomb"`: Enables tracing to [honeycomb.io].
    - Env variable `HONEYCOMBIO_WRITE_KEY` (required).
    - Env variable `TRACELEVEL`, sets the tracing level filter, defaults to `info`.
    - Writes to a dataset named `{service_name}-{environment}`.
        - `service_name` is from `preroll::main!("service_name", ...)`.
        - `environment` is from `ENVIRONMENT`, or defaults to `"development"`.
- `"postgres"`: Enables a postgres connection pool with transactions.
    - Env variable `PGURL`, which should be a properly formatted `postgres://` database url.
        - Defaults to `"postgres://localhost/{service_name}"` (default postgres port).
        - `service_name` is from `preroll::main!("service_name", ...)`.
    - Env variable `PGMAXCONNECTIONS`, default 5 connections.
    - Enables [`PostgresRequestExt`][prelude::PostgresRequestExt] and [`test_utils::create_client_and_postgres`][].

#### List of other optional features:
- `"panic-on-error"`: Makes the response logger [panic][] on error rather than log.
    - Do not use in production. Prevents `--release` compilation.

### General Environment Settings
The following environment variables are read during `preroll::main!`:
- `ENVIRONMENT`: If this starts with `prod`, load the production-mode JSON logger, avoid `.env`.
- `FORCE_DOTENV`: Override production-mode, force-load environment from `.env`.
- `HOST`: Sets the hostname that this service will listen on. Defaults to `"127.0.0.1"`.
- `LOGLEVEL`: Set the logger's level filter, defaults to `info` in production-mode, `debug` in development-mode.
- `PORT`: Sets the port that this service will listen on. Defaults to `8080`.

### Note:

This crate is intentionally somewhat prescriptive in how it templates a service and the interaction with
add-on features such as Postgres (via [SQLx][]).

[`preroll::main!`]: https://docs.rs/preroll/0.2.0/preroll/macro.main.html
[`preroll::prelude::*;`]: https://docs.rs/preroll/0.2.0/preroll/prelude/index.html
[`JsonError`]: https://docs.rs/preroll/0.2.0/preroll/struct.JsonError.html
[async-std]: https://async.rs/
[honeycomb.io]: https://www.honeycomb.io/
[SQLx]: https://github.com/launchbadge/sqlx#sqlx
[Surf]: https://github.com/http-rs/surf#surf
[Test utils]: https://docs.rs/preroll/0.2.0/preroll/test_utils/index.html
[Tide]: https://github.com/http-rs/tide#tide

## API Reference

[API Reference on Docs.rs](https://docs.rs/preroll/0.2.0/preroll/#modules)

## License

Licensed under the [BlueOak Model License 1.0.0](LICENSE.md) — _[Contributions via DCO 1.1](contributing.md#developers-certificate-of-origin)_
