use preroll::test_utils::{self, TestResult};
use tide::{Request, Server};

/// Creates a test application with routes and mocks set up,
/// and hands back a client which is already connected to the server.
pub async fn create_client() -> TestResult<surf::Client> {
    let google_client = test_utils::mock_client("http://google.com/", setup_example_mocks);

    let state = preroll_example::State { google_client };

    test_utils::create_client(
        state,
        (
            preroll_example::setup_routes_v1,
            preroll_example::setup_routes_v2,
        ),
    )
    .await
}

pub fn setup_example_mocks(google_mock: &mut Server<()>) {
    google_mock
        .at("/")
        .get(|_req: Request<()>| async { Ok(tide::StatusCode::MovedPermanently) });
}
