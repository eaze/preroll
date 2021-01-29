use surf::Url;

use preroll_main_test::{setup_custom, setup_routes_v1, setup_routes_v2, AppState};

preroll::main!(
    "preroll-main-test",
    setup_app_state,
    setup_custom,
    setup_routes_v1,
    setup_routes_v2
);

pub async fn setup_app_state() -> preroll::SetupResult<AppState> {
    let mut google_client = surf::client();
    google_client.set_base_url(Url::parse("http://example.org/")?);

    Ok(AppState { google_client })
}
