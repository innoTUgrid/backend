use crate::models::PingResponse;

use crate::tests::test_util::get_client;

#[tokio::test]
async fn test_ping() {
    // Setup
    let client = get_client().await;

    // Send a request to the ping endpoint
    let response = client.get("/v1/").send().await;

    // Verify the response
    assert!(response.status().is_success());
    let body: PingResponse = response.json().await;
    assert_eq!(body.message, "0xDECAFBAD");
}
