use preroll::test_utils::assert_json_error;

mod test_utils;

// This is how it is suggested to write unit-integration tests with preroll.

#[async_std::test]
async fn test_preroll_main_test_utils() {
    let client = test_utils::create_client().await.unwrap();

    {
        let response = client
            .get("/api/v1/test-preroll-setup-routes")
            .recv_string()
            .await
            .unwrap();

        assert_eq!(response, "preroll successfully set route in v1");
    }

    {
        let mut response = client.get("/api/v1/test-client-error").await.unwrap();

        assert_json_error(
            &mut response,
            400,
            "failed with reason: missing field `param`",
        )
        .await;
    }

    {
        let response = client.get("/api/v2/fetch-example").await.unwrap();

        assert_eq!(response.status(), 301);
    }

    // Part of https://github.com/eaze/preroll/pull/9
    // {
    //     let response = client.get("/monitor/ping").recv_string().await.unwrap();

    //     assert_eq!(response, "preroll-example");
    // }

    // TODO(Jeremiah): Should tests also have this?
    // #[cfg(debug_assertions)]
    // {
    //     let mut response = client.get("/internal-error").await.unwrap();

    //     assert_json_error(
    //         &mut response,
    //         500,
    //         "Internal Server Error (correlation_id=00000000-0000-0000-0000-000000000000)",
    //     )
    //     .await;
    // }
}
