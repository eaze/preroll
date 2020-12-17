use std::sync::Arc;

use tide::Server;
struct AppState {}

async fn setup_app_state() -> preroll::setup::Result<AppState> {
    Ok(AppState {})
}

fn setup_routes(server: &mut Server<Arc<AppState>>) {
    server
        .at("test-preroll-setup-routes")
        .get(|_| async { Ok("preroll successfully set route") });
}

preroll::main!("preroll-main-test", setup_app_state, setup_routes);
