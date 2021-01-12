use std::sync::Arc;

use preroll::SetupResult;
use tide::{Request, Route, Server};

struct AppState {}

async fn setup_app_state() -> SetupResult<AppState> {
    Ok(AppState {})
}

async fn setup_custom(server: Server<Arc<AppState>>) -> SetupResult<Server<Arc<AppState>>> {
    Ok(server)
}

#[derive(serde::Deserialize)]
struct Query {
    pub param: u16,
}

async fn get_client_error(req: Request<Arc<AppState>>) -> tide::Result<&'static str> {
    let _query: Query = req.query()?;
    Ok("Should error")
}

fn setup_routes_v1(mut server: Route<'_, Arc<AppState>>) {
    server
        .at("/test-preroll-setup-routes")
        .get(|_| async { Ok("preroll successfully set route in v1") });

    server.at("/test-client-error").get(get_client_error);
}

fn setup_routes_v2(mut server: Route<'_, Arc<AppState>>) {
    server
        .at("/test-preroll-setup-routes")
        .get(|_| async { Ok("preroll successfully set route in v2") });
}

preroll::main!(
    "preroll-main-test",
    setup_app_state,
    setup_custom,
    setup_routes_v1,
    setup_routes_v2
);
