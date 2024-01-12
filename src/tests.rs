use crate::infrastructure::create_connection_pool;
use crate::infrastructure::create_router;

use crate::models::{
    Datapoint, MetaInput, MetaOutput, MetaRows, PingResponse, ResampledTimeseries,
};

use crate::models::Timeseries;
use crate::models::{NewDatapoint, TimeseriesBody};
use axum_test_helper::TestClient;
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;
use time::OffsetDateTime;

fn get_random_string(size: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), size)
}

async fn get_client() -> TestClient {
    let pool = create_connection_pool().await;
    let router = create_router(pool);

    TestClient::new(router)
}

async fn add_meta(client: &TestClient, identifier: &str) -> MetaOutput {
    let meta = MetaInput {
        identifier: identifier.to_string(),
        unit: String::from("testUnit"),
        carrier: Some(String::from("oil")),
        consumption: Some(true),
    };
    let res = client.post("/v1/meta/").json(&meta).send().await;
    assert!(res.status().is_success());

    let r: MetaOutput = res.json().await;
    assert_eq!(r.identifier, identifier);
    r
}

async fn add_timeseries(
    client: &TestClient,
    identifier: &str,
    value: f64,
) -> TimeseriesBody<Datapoint> {
    let timeseries = NewDatapoint {
        timestamp: OffsetDateTime::now_utc(),
        value,
        identifier: identifier.to_string(),
    };
    let res = client
        .post("/v1/ts/")
        .json(&TimeseriesBody { timeseries })
        .send()
        .await;
    assert!(res.status().is_success());

    let r: TimeseriesBody<Datapoint> = res.json().await;
    assert_eq!(r.timeseries.value, value);
    r
}

#[tokio::test]
async fn test_add_timeseries_bad_data() {
    let client = get_client().await;
    let identifier = get_random_string(10);
    add_meta(&client, &identifier).await;

    let rfc_3339_format = &time::format_description::well_known::Rfc3339;
    let timeseries = json!({
        "timestamp": OffsetDateTime::now_utc().format(rfc_3339_format).unwrap(),
        "value": 42,
        "wrongKey": identifier.to_string(),
    });
    let response = client
        .post("/v1/ts/")
        .json(&TimeseriesBody { timeseries })
        .send()
        .await;
    assert!(response.status().is_client_error());
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

#[tokio::test]
async fn test_add_timeseries() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
}

#[tokio::test]
async fn test_get_timeseries_by_identifier() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;

    let response = client.get(&format!("/v1/ts/{}/", identifier)).send().await;
    assert!(response.status().is_success());

    let body: Timeseries = response.json().await;
    assert_eq!(body.meta.identifier, identifier);
    assert_eq!(body.datapoints.len(), 1);
}

#[tokio::test]
async fn test_get_timeseries_by_identifier_from_filter() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;

    let response = client
        .get(&format!("/v1/ts/{}/?from=2022-11-29T09:31:51Z", identifier))
        .send()
        .await;
    assert!(response.status().is_success());
    let body: Timeseries = response.json().await;
    assert_eq!(body.meta.identifier, identifier);
    assert_eq!(body.datapoints.len(), 1);
}

#[tokio::test]
async fn test_get_timeseries_by_identifier_to_filter() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;

    let response = client
        .get(&format!("/v1/ts/{}/?to=2022-11-29T09:31:51Z", identifier))
        .send()
        .await;
    assert!(response.status().is_success());
    let body: Timeseries = response.json().await;
    assert_eq!(body.meta.identifier, identifier);
    assert_eq!(body.datapoints.len(), 0);
}

#[tokio::test]
async fn test_get_timeseries_by_identifier_with_ts_filter() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    let response = client
        .get(&format!(
            "/v1/ts/{}/?from=2022-11-29T09:31:51Z&to=2022-12-01T00:00:00Z",
            identifier
        ))
        .send()
        .await;
    assert!(response.status().is_success());
    let body: Timeseries = response.json().await;
    assert_eq!(body.meta.identifier, identifier);
    assert_eq!(body.datapoints.len(), 0);
}

#[tokio::test]
async fn test_resample_timeseries_by_identifier() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    add_timeseries(&client, &identifier, 66.0).await;

    let response = client
        .get(&format!("/v1/ts/{}/resample?interval=1hour", identifier))
        .send()
        .await;
    assert!(response.status().is_success());

    let body: ResampledTimeseries = response.json().await;
    assert_eq!(body.datapoints.first().unwrap().mean_value.unwrap(), 54.0);
}
#[tokio::test]
async fn test_resample_timeseries_by_identifier_with_ts_filter_from() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    add_timeseries(&client, &identifier, 66.0).await;

    let response = client
        .get(&format!(
            "/v1/ts/{}/resample?interval=1hour&from=2022-11-29T09:31:51Z",
            identifier
        ))
        .send()
        .await;
    assert!(response.status().is_success());

    let body: ResampledTimeseries = response.json().await;
    assert_eq!(body.datapoints.first().unwrap().mean_value.unwrap(), 54.0);
}
#[tokio::test]
async fn test_resample_timeseries_by_identifier_with_ts_filter_to() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    add_timeseries(&client, &identifier, 66.0).await;

    let response = client
        .get(&format!(
            "/v1/ts/{}/resample?interval=1hour&to=2022-11-29T09:31:51Z",
            identifier
        ))
        .send()
        .await;
    assert!(response.status().is_success());

    let body: ResampledTimeseries = response.json().await;
    assert_eq!(body.datapoints.len(), 0)
}
#[tokio::test]
async fn test_resample_timeseries_by_identifier_with_ts_filter() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    add_timeseries(&client, &identifier, 66.0).await;

    let response = client
        .get(&format!(
            "/v1/ts/{}/resample?interval=1hour&from=2022-11-29T09:31:51Z&to=2022-12-01T00:00:00Z",
            identifier
        ))
        .send()
        .await;
    assert!(response.status().is_success());

    let body: ResampledTimeseries = response.json().await;
    assert_eq!(body.datapoints.len(), 0)
}
#[tokio::test]
async fn test_resample_timeseries_by_identifier_bad_ts_filter() {
    let client = get_client().await;
    let identifier = get_random_string(10);

    add_meta(&client, &identifier).await;
    add_timeseries(&client, &identifier, 42.0).await;
    add_timeseries(&client, &identifier, 66.0).await;

    let response = client
        .get(&format!(
            "/v1/ts/{}/resample?interval=1hour&from=23542365346747&to=2022-12-01T00:00:00Z",
            identifier
        ))
        .send()
        .await;
    assert!(response.status().is_client_error());
}
