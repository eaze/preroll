# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.2] - 2021-07-12

### Fixes
- `lambda-http`: This feature flag now actually works. (Oops.)

## [0.8.1] - 2021-07-12

### Fixes
- `lambda-http`: No longer disables the logging middleware.

## [0.8.0] - 2021-06-15

### Additions
- New `"lambda-http"` feature, which changes the HTTP listener to connect to an AWS Lambda execution environment.
    - Is no longer reachable as a regular http server, but accepts http lambda requests as if it were one.
    - Some environment variables, such as `PORT`, are disregarded.
    - If the `"honeycomb"` feature is enabled, trace events are written to stdout, and must be collected via
        a layer provided by Honeycomb. See: https://docs.honeycomb.io/getting-data-in/integrations/aws/aws-lambda/

## [0.7.0] - 2021-05-19

### Changes
- The default backend for the included [Surf][] http client has changed from libcurl (via [Isahc][]) to [http-client's async-h1 client][].
- `honeycomb`: Environemtn variables now start with `HONEYCOMB_` rather than `HONEYCOMBIO_`.
    - Consistency with other honeycomb SDK's.
- `honeycomb`: `HONEYCOMBIO_WRITE_KEY` is now `HONEYCOMB_WRITEKEY` (required).
    - Consistency with other honeycomb SDK's.

### Additions
- `honeycomb`: Added `HONEYCOMB_SAMPLE_RATE` environment var usage.

[http-client's async-h1 client]: https://github.com/http-rs/http-client/tree/main/src/h1
[Isahc]: https://github.com/sagebind/isahc
[Surf]: https://github.com/http-rs/surf

## [0.6.0] - 2021-04-15

### Additions
- `postgres`: Added a `PGMAXLIFETIME` environemnt variable, set in minutes.

### Dependencies
- `honeycomb`: Switched back to `tracing-honeycomb` and upgraded to `0.3`. 
   - This allows patch upgrades to be picked up via `cargo update`, and is otherwise identical to `0.2.1-eaze.7`.

## [0.5.7] - 2021-04-05

- `honeycomb`: Added events for generic Request/Response http properties.
- Changed the ordering of json log fields to be better for plaintext viewers.
- Fixed hostname resolution in `/monitor/status`.

## [0.5.6] - 2021-03-29

- Dependency upgrade to eaze-tracing-honeycomb 0.2.1-eaze.7
    - `honeycomb`: This fixes a deadlock. See [this commit] for details.

[this commit]: https://github.com/eaze/tracing-honeycomb/commit/9dd18b55ea96b95ce76d0051dbcbd085b7e7f2f1

## [0.5.5] - 2021-03-24

- `honeycomb`: Fixed timestamps to correctly be from spans/events.
- Dependency upgrade to eaze-tracing-honeycomb 0.2.1-eaze.6

## [0.5.4] - 2021-03-24

- `honeycomb`: Sub-millisecond "duration_ms" using f64.
- Dependency upgrade to eaze-tracing-honeycomb 0.2.1-eaze.5

## [0.5.3] - 2021-03-22

- Enables Postgres trace events.
- Dependency upgrade to tide-sqlx 0.6.0

## [0.5.2] - 2021-03-19

- Dependency upgrade to eaze-tracing-honeycomb 0.2.1-eaze.4

## [0.5.1] - 2021-03-09

- Reduce dependency footprint from async-std.
    - Avoids wasm-related features.

## [0.5.0] - 2021-03-09

- Dependency upgrade to sqlx 0.5 & tide-sqlx 0.5 for the `postgres` feature.

## [0.4.3] - 2021-02-22

- Fixes `X-Honeycomb-Trace` header parsing.
    - Under the hood, this meant a switch to [the Eaze fork of tracing-honeycomb](https://github.com/eaze/tracing-honeycomb).

## [0.4.2] - 2021-02-16

- Avoid causing http errors on invalid `X-Honeycomb-Id` and `X-Request-Id`.
    - Instead, these get logged.

## [0.4.1] - 2021-02-02

- Updated the `"honeycomb"` feature to respect the `"HONEYCOMBIO_DATASET"` environment variable when possible.

## [0.4.0] - 2021-02-01

The same as 0.3 but with a forgotten update to Tide 0.16.

- Dependency upgrade to Tide 0.14 & tide-sqlx 0.4.

## [0.3.0] - 2021-02-01

- Removed the `"custom_middleware"` feature, is now automatically part of the optional arguments for `preroll::main!`.
- Changed `preroll::main!` to accept `VariadicRoutes` instead of variable macro arguments.
- Changed `preroll::test_utils::create_client` to accept `VariadicRoutes`, enabled api versioned testing.
- Changed assertions to always unwrap and panic immediately rather than return a `Result`.
- Changed assertions to accept `AsMut<http_types::Response>` rather than `surf::Response`.
- Added more test assertion helpers to `preroll::test_utils`.
- Added `/monitor/status` built-in endpoint to expose useful status information.
- Updated all documentation with major additions, overhauls, and examples.
- Improved the standalone example / test subproject.
- Improved internal CI.

## [0.2.0] - 2021-01-12

- Changed the api mounting point to be versioned - e.g. `/api/v1`.
    - This is based on argument position of the route handlers provided to `preroll::main!`.
- Changed routes setup function to instead receive `Route<'_, Arc<AppState>>`.
- Added `"test"` feature (makes UUIDs such as `correlation_id` be constant an nil).
- Added `"custom_middleware"` feature to add an extra hook into `preroll::main!`.
- Added an always-enabled `GET /monitor/ping` which responds only with `service_name`.
    - This is excluded from middleware.
- Added a debug-mode-only `GET /internal-error` for easy testing.
- Exposed and documented `JsonError`.
- Improved http error output for `assert_json_error`.

## [0.1.2] - 2021-01-06

- Fixed `honeycomb` feature. _(Incorrect `cfg` statements.)_

## [0.1.1] - 2020-12-21

- Fixed missing/empty `Cargo.toml` fields.

## [0.1.0] - 2020-12-21

- Initial exports:
  - `prelude`
  - `test_utils`
  - `utils`
  - `main`
  - `SetupResult`

- Initial feature set:
  - `"honeycomb"`
  - `"postgres"`
  - `"panic-on-error"`

Note: the following override is currently required to make `libhoney-rust` use `async-std` as its runtime:
```toml
[patch.crates-io.libhoney-rust]
git = "https://github.com/eaze/libhoney-rust.git"
branch = "runtime-async-std"
optional = true
default-features = false
features = ["runtime-async-std"]
```
