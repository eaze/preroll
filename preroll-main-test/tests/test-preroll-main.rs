use std::time::Duration;

use assert_cmd::cargo::cargo_bin;
use async_std::process::{Command, Stdio};
use async_std::task::{sleep, spawn};
use futures_lite::future::race;
use portpicker::pick_unused_port;
use preroll::test_utils::assert_json_error;

#[async_std::test]
async fn test_preroll_main() {
    let cargo_bin_path = cargo_bin("preroll-main-test");
    let port = pick_unused_port().unwrap_or(8080).to_string();

    let mut server_proc = Command::new(cargo_bin_path)
        .env("HOST", "127.0.0.1")
        .env("PORT", &port)
        .kill_on_drop(true)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();

    let client_thread = spawn(async move {
        sleep(Duration::from_millis(250)).await;

        {
            let url = format!("http://127.0.0.1:{}/test-preroll-setup-routes", port);
            let response = surf::get(url).recv_string().await.unwrap();

            assert_eq!(response, "preroll successfully set route");
        }

        {
            let url = format!("http://127.0.0.1:{}/internal-error", port);
            let mut response = surf::get(url).await.unwrap();

            assert_json_error(
                &mut response,
                500,
                "Internal Server Error (correlation_id=00000000-0000-0000-0000-000000000000)",
            )
            .await
            .unwrap();
        }

        // {
        //     let url = format!("http://127.0.0.1:{}/monitor/ping", port);
        //     let response = surf::get(url).recv_str().await?;
        //
        //     assert_eq!(response, "preroll-main-test")
        // }
    });

    let a = async {
        server_proc.status().await.unwrap();
    };

    let b = async {
        client_thread.await;
    };

    race(a, b).await;
}
