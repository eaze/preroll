use std::sync::Arc;

use tide::{Request, Server};
struct AppState {}

async fn setup_app_state() -> preroll::SetupResult<AppState> {
    Ok(AppState {})
}

async fn get_internal_error(_req: Request<Arc<AppState>>) -> tide::Result<&'static str> {
    Err(tide::Error::from_str(
        500,
        "Intentional Server Error from GET /internal-error",
    ))
}

fn setup_routes(server: &mut Server<Arc<AppState>>) {
    server.at("/internal-error").get(get_internal_error);

    server
        .at("/test-preroll-setup-routes")
        .get(|_| async { Ok("preroll successfully set route") });
}

preroll::main!("preroll-main-test", setup_app_state, setup_routes);
