use crate::tests::test_util::get_client;

use crate::models::KpiResult;

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
