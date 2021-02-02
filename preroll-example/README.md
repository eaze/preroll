# preroll example

A working example service built with preroll with tests that use [`preroll::test_utils`][].

- `lib.rs`: Service code that you may want to test with [`preroll::test_utils`][].
- `main.rs`: The production / debug running binary entry point.
- `tests/test_utils.rs`: Project-local test utility wrappers for actual tests.
  - Sets up the service's routes and mocks out any external client APIs.
- `tests/integration.rs`: Tests for the example service's `lib.rs`.

[`preroll::test_utils`]: https://docs.rs/preroll/0.3.0/preroll/test_utils/index.html
