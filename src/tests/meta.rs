use crate::models::MetaRows;

use crate::tests::test_util::add_meta;
use crate::tests::test_util::get_client;
use crate::tests::test_util::get_random_string;

use serde_json::json;

#[tokio::test]
async fn test_add_meta_bad_data() {
    let client = get_client().await;

    let bad_metadata = json!(
        {
            "identifier": "testIdentifier",
            "unitSchmunit": "testUnit",
            "carrier": 42
        }
    );
    let response = client.post("/v1/meta/").json(&bad_metadata).send().await;
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_read_meta() {
    let client = get_client().await;

    let identifier = get_random_string(10);
    let meta = add_meta(&client, &identifier).await;

    let response = client.get("/v1/meta/").send().await;
    assert!(response.status().is_success());

    let body: MetaRows = response.json().await;

    assert!(
        body.values.iter().any(|x| x.identifier == meta.identifier),
        "identifier not found in response"
    );
}
