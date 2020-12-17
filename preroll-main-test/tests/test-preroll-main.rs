use std::time::Duration;

use assert_cmd::cargo::cargo_bin;
use async_std::process::Command;
use async_std::task::sleep;
use portpicker::pick_unused_port;

#[async_std::test]
async fn test_preroll_main() {
    let cargo_bin_path = cargo_bin("preroll-main-test");
    let port = pick_unused_port().unwrap_or(8080).to_string();

    let mut server_proc = Command::new(cargo_bin_path)
        .env("HOST", "127.0.0.1")
        .env("PORT", &port)
        .spawn()
        .unwrap();

    sleep(Duration::from_millis(100)).await;

    {
        let url = format!("http://127.0.0.1:{}/test-preroll-setup-routes", port);
        let response = surf::get(url).recv_string().await.unwrap();

        assert_eq!(response, "preroll successfully set route")
    }

    // {
    //     let url = format!("http://127.0.0.1:{}/monitor/ping", port);
    //     let response = surf::get(url).recv_str().await?;

    //     assert_eq!(response, "preroll-main-test")
    // }

    server_proc.kill().unwrap();
}
