use crate::models::{EmissionFactor};
use crate::tests::test_util::get_client;

#[tokio::test]
pub async fn test_get_emission_factors() {
    let client = get_client().await;

    let response = client.get("/v1/emission_factors/").send().await;
    assert!(response.status().is_success());

    let body: Vec<EmissionFactor> = response.json().await;
    assert_eq!(
        body.iter().find(|&x| x.carrier == "coal").unwrap().factor,
        0.82
    );
}
