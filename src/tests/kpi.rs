use crate::tests::test_util::get_client;

use crate::models::{EmissionsByCarrier, KpiResult};

#[tokio::test]
async fn test_kpi_self_consumption() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/self_consumption/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z")
        .send()
        .await;

    assert!(response.status().is_success());
    let body: KpiResult = response.json().await;
    assert_eq!(body.value, 1.0);
}

#[tokio::test]
async fn test_kpi_autarky() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/autarky/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z")
        .send()
        .await;

    assert!(response.status().is_success());
    let body: KpiResult = response.json().await;
    assert_eq!((body.value * 10.0).floor(), 4.0);
}

#[tokio::test]
async fn test_get_total_consumption() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/total_consumption/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_get_total_co2_emissions() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/total_co2_emissions/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_get_scope_one_emissions() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/scope_one_emissions/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response.status().is_success());
    let body: Vec<EmissionsByCarrier> = response.json().await;
    body.iter().for_each(|x| {
        if x.carrier_name == "coal" {
            assert_eq!(x.value, 2.3074296675191817);
        }
    });
}

#[tokio::test]
async fn test_get_scope_two_emissions() {
    let client = get_client().await;

    let response = client
        .get("/v1/kpi/scope_two_emissions/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response.status().is_success());
    let body: Vec<EmissionsByCarrier> = response.json().await;
    body.iter().for_each(|x| {
        if x.carrier_name == "coal" {
            assert_eq!(x.value, 2.3074296675191817);
        }
    });
}

#[tokio::test]
async fn test_scope_one_plus_two_eq_total() {
    let client = get_client().await;

    let response_scope_one = client
        .get("/v1/kpi/scope_two_emissions/?from=2019-01-01T12:00:00:000Z&to=2019-02-01T12:00:00:000Z&interval=1hour")
        .send()
        .await;

    assert!(response_scope_one.status().is_success());
    let body_scope_one: Vec<EmissionsByCarrier> = response_scope_one.json().await;

    let response_scope_two = client
        .get("/v1/kpi/scope_one_emissions/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response_scope_two.status().is_success());
    let body_scope_two: Vec<EmissionsByCarrier> = response_scope_two.json().await;

    let response = client
        .get("/v1/kpi/total_co2_emissions/?from=2019-01-01T12:00:00Z&to=2019-02-01T12:00:00Z&interval=1hour")
        .send()
        .await;

    assert!(response.status().is_success());
    let body: KpiResult = response.json().await;

    let sum_scope_one = body_scope_one.iter().fold(0.0, |acc, x| acc + x.value);
    let sum_scope_two = body_scope_two.iter().fold(0.0, |acc, x| acc + x.value);

    assert_eq!(sum_scope_one + sum_scope_two, body.value);
}
