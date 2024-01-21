use crate::tests::test_util::get_client;
use serde_json::{json, Value};

#[tokio::test]
async fn test_config() {
    let client = get_client().await;

    let config = json!(
    {
      "test": {
        "test": "test"
      }
    });

    let response = client.post("/v1/config/").json(&config).send().await;

    let r: Value = response.json().await;
    assert_eq!(r, config);

    let response = client.get("/v1/config/").send().await;

    let r: Value = response.json().await;
    assert_eq!(r, config);
}
