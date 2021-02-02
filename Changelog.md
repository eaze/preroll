# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2021-01-01

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
