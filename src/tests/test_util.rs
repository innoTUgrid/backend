use crate::app_config::AppConfig;
use crate::infrastructure::create_connection_pool;
use crate::infrastructure::create_router;

use crate::models::{Datapoint, MetaInput, MetaOutput};

use crate::models::{NewDatapoint, TimeseriesBody};
use axum_test_helper::TestClient;
use rand::distributions::{Alphanumeric, DistString};

use time::OffsetDateTime;

pub fn get_random_string(size: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), size)
}

pub async fn get_client() -> TestClient {
    let config = AppConfig::new();
    let pool = create_connection_pool(&config).await;
    let router = create_router(pool, &config);

    TestClient::new(router)
}

pub async fn add_meta(client: &TestClient, identifier: &str) -> MetaOutput {
    let meta = MetaInput {
        identifier: identifier.to_string(),
        unit: String::from("testUnit"),
        carrier: Some(String::from("oil")),
        consumption: Some(true),
        description: Some("description".to_string()),
        local: Some(true),
    };
    let res = client.post("/v1/meta/").json(&meta).send().await;
    assert!(res.status().is_success());

    let r: MetaOutput = res.json().await;
    assert_eq!(r.identifier, identifier);
    r
}

pub async fn add_timeseries(
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
        .json(&TimeseriesBody {
            timeseries: Vec::from([timeseries]),
        })
        .send()
        .await;
    assert!(res.status().is_success());

    let r: TimeseriesBody<Datapoint> = res.json().await;
    assert_eq!(r.timeseries[0].value, value);
    r
}
