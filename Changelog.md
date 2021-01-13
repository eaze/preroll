# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
