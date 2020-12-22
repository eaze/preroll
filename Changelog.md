# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
