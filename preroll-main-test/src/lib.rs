use std::sync::Arc;

use preroll::SetupResult;
use tide::{http, Request, Response, Route, Server};

pub struct AppState {
    pub google_client: surf::Client,
}

pub async fn setup_custom(server: Server<Arc<AppState>>) -> SetupResult<Server<Arc<AppState>>> {
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

async fn fetch_example(req: Request<Arc<AppState>>) -> tide::Result<Response> {
    let state = req.state();

    let res: http::Response = state.google_client.get("http://google.com").await?.into();
    Ok(res.into())
}

pub fn setup_routes_v1(mut server: Route<'_, Arc<AppState>>) {
    server
        .at("/test-preroll-setup-routes")
        .get(|_| async { Ok("preroll successfully set route in v1") });

    server.at("/test-client-error").get(get_client_error);
}

pub fn setup_routes_v2(mut server: Route<'_, Arc<AppState>>) {
    server.at("/fetch-example").get(fetch_example);
}
