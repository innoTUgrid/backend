use crate::models::{MetaInput, MetaOutput, MetaRows};

use crate::tests::test_util::add_meta;
use crate::tests::test_util::get_client;
use crate::tests::test_util::get_random_string;

use serde_json::json;

#[tokio::test]
async fn test_add_meta() {
    let client = get_client().await;

    let identifier = get_random_string(10);
    let meta = add_meta(&client, &identifier).await;

    assert_eq!(meta.identifier, identifier);
}

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
async fn test_get_meta_by_identifier() {
    let client = get_client().await;
    let identifier = get_random_string(10);
    let meta = add_meta(&client, &identifier).await;
    assert_eq!(meta.identifier, identifier);
    let response = client
        .get(format!("/v1/meta/{}/", identifier).as_ref())
        .send()
        .await;
    assert!(response.status().is_success());
    let body = response.json::<MetaOutput>().await;
    assert_eq!(body.identifier, identifier);
}

#[tokio::test]
async fn test_read_meta() {
    let client = get_client().await;

    let identifier = get_random_string(10);
    let meta = add_meta(&client, &identifier).await;
    assert_eq!(meta.identifier, identifier);

    let response = client.get("/v1/meta/").send().await;
    assert!(response.status().is_success());

    let body: MetaRows = response.json().await;
    assert!(
        body.values.iter().any(|x| x.identifier == meta.identifier),
        "identifier not found in response"
    );
}

#[tokio::test]
async fn test_add_meta_empty_carrier() {
    let client = get_client().await;

    let identifier = get_random_string(10);

    let meta = MetaInput {
        identifier: identifier.to_string(),
        unit: String::from("testUnit"),
        carrier: None,
        consumption: Some(true),
    };
    let res = client.post("/v1/meta/").json(&meta).send().await;
    assert!(res.status().is_success());

    let r: MetaOutput = res.json().await;
    assert_eq!(r.identifier, identifier);
    assert!(r.carrier.is_none());
}
