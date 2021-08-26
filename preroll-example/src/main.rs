use std::convert::TryInto;

use surf::{Client, Config, Url};

use preroll_example::{setup_custom, setup_routes_v1, setup_routes_v2, State};

preroll::main!(
    "preroll-example",
    setup_app_state,
    setup_custom,
    (setup_routes_v1, setup_routes_v2)
);

pub async fn setup_app_state() -> preroll::SetupResult<State> {
    let google_client: Client = Config::new()
        .set_base_url(Url::parse("http://example.org/")?)
        .try_into()?;

    Ok(State { google_client })
}
