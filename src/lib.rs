#![forbid(unsafe_code, future_incompatible)]
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

pub mod logging;
pub mod middleware;
pub mod setup;
pub mod utils;

#[macro_export]
macro_rules! main {
    // preroll::main!("service-name", routes_setup_function);
    ($service_name:tt, $routes_setup:tt) => {
        $crate::main!(service_name, (), async { () }, routes_setup);
    };

    // preroll::main!("service-name", state_setup_function, routes_setup_function);
    ($service_name:tt, $state_setup:tt, $routes_setup:tt) => {
        fn main() -> preroll::setup::SetupResult<()> {
            preroll::setup::block_on(async {
                preroll::setup::initial_setup($service_name)?;

                let state = $state_setup().await?;

                let mut server = preroll::setup::setup_middleware($service_name, state).await?;

                $routes_setup(&mut server);

                preroll::setup::start_server(server).await?;

                Ok(())
            })
        }
    };
}
