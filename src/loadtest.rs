use goose::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::time::Duration;
use time::OffsetDateTime;


#[allow(dead_code)]
async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_autarky(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/kpi/autarky/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_cost_savings(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/kpi/cost_savings/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_co2_savings(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/kpi/co2_savings/?interval=1hour").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_consumption(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/kpi/consumption/?interval=1month").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_co2_emissions(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/v1/kpi/total_co2_emissions/?interval=1month")
        .await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_scope_one_emissions(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/v1/kpi/scope_one_emissions/?interval=1month")
        .await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_scope_two_emissions(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/v1/kpi/scope_two_emissions/?interval=1month")
        .await?;
    Ok(())
}

#[allow(dead_code)]
async fn loadtest_get_specific_metadata(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/meta/smard_market_price/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_get_all_metadata(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/meta/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_get_specific_timeseries(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/ts/smard_market_price/").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_get_resampled_timeseries(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/v1/ts/smard_market_price/resample/?interval=1hour")
        .await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_get_emission_factor(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/v1/emission_factors/?source=IPCC").await?;
    Ok(())
}
#[allow(dead_code)]
async fn loadtest_post_random_meta(user: &mut GooseUser) -> TransactionResult {
    let identifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let unit: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let description: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let meta = &serde_json::json!({
                "identifier": identifier,
                "unit": unit,
                "description": description,
    });
    let _response = user.post_json("/v1/meta/", meta).await?;

    let datapoint = rand::thread_rng().gen_range(0.0..1000.0);
    let random_days = rand::thread_rng().gen_range(0..365);
    let random_hours = rand::thread_rng().gen_range(0..24);
    let random_minutes = rand::thread_rng().gen_range(0..60);
    let random_seconds = rand::thread_rng().gen_range(0..60);

    let timestamp = OffsetDateTime::now_utc()
        - Duration::from_secs(random_days * random_hours * random_minutes * random_seconds);

    let timestamp_str = timestamp
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();

    let ts = &serde_json::json!({
        "timeseries":[
            {
                "identifier": identifier,
                "timestamp": timestamp_str,
                "value": datapoint
            }
        ]
    });
    let _response = user.post_json("/v1/ts/", ts).await?;
    Ok(())
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("read")
                .register_transaction(transaction!(loadtest_co2_savings))
                .register_transaction(transaction!(loadtest_cost_savings))
                .register_transaction(transaction!(loadtest_scope_one_emissions))
                .register_transaction(transaction!(loadtest_scope_two_emissions))
                .register_transaction(transaction!(loadtest_get_all_metadata))
                .register_transaction(transaction!(loadtest_get_specific_timeseries)),
        )
        .register_scenario(
            scenario!("ingestion").register_transaction(transaction!(loadtest_post_random_meta)),
        )
        .execute()
        .await?;

    Ok(())
}
