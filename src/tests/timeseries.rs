use crate::models::ResampledTimeseries;

use crate::models::Timeseries;
use crate::models::TimeseriesBody;
use crate::tests::test_util::add_meta;
use crate::tests::test_util::add_timeseries;
use crate::tests::test_util::get_client;
use crate::tests::test_util::get_random_string;

use serde_json::json;
use time::OffsetDateTime;

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
        .json(&TimeseriesBody { timeseries: Vec::from([timeseries]) })
        .send()
        .await;
    assert!(response.status().is_client_error());
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
