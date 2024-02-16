use crate::error::ApiError;
use anyhow::anyhow;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::postgres::types::PgInterval;
use sqlx::{Pool, Postgres};
use std::fmt::Formatter;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// wrap postgres timestamptz to achieve human-readable serialization
#[derive(sqlx::Type)]
pub struct Timestamptz(pub OffsetDateTime);

/// simplify return types for axum handlers with this wrapper
pub type Result<T, E = ApiError> = std::result::Result<T, E>;

impl Serialize for Timestamptz {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0.format(&Rfc3339).map_err(serde::ser::Error::custom)?)
    }
}
impl<'de> Deserialize<'de> for Timestamptz {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;
        impl Visitor<'_> for StrVisitor {
            type Value = Timestamptz;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                f.pad("expected string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                OffsetDateTime::parse(v, &Rfc3339)
                    .map(Timestamptz)
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, sqlx::FromRow)]
pub struct TimeseriesMeta {
    pub id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
    pub description: Option<String>,
    pub local: Option<bool>,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Datapoint {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub value: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
/// TimescaleDB's `time_bucket` function returns a nullable column
pub struct ResampledDatapoint {
    #[serde(with = "time::serde::rfc3339::option")]
    pub bucket: Option<OffsetDateTime>,
    pub mean_value: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timeseries {
    pub datapoints: Vec<Datapoint>,
    pub meta: TimeseriesMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResampledTimeseries {
    pub datapoints: Vec<ResampledDatapoint>,
    pub meta: TimeseriesMeta,
}

/// Intermediate representation for join tables from the database.
#[derive(Debug, Serialize)]
pub struct DatapointWithMetadata {
    pub id: i64,
    pub timestamp: OffsetDateTime,
    pub value: f64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub meta_id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct MultipleDatapointsWithMetadata {
    pub datapoints: Vec<DatapointWithMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewDatapoint {
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub value: f64,
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeseriesBody<T = Timeseries> {
    pub timeseries: Vec<T>,
}
#[derive(Debug, Clone, Serialize)]
pub struct SingleMetaResponse {
    pub metadata: TimeseriesMeta,
}
#[derive(Debug, Clone, Serialize)]
pub struct ManyMetaResponse {
    pub data: Vec<TimeseriesMeta>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PingResponse {
    pub message: String,
}

impl PingResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
impl Default for PingResponse {
    fn default() -> Self {
        Self::new(String::from("0xDECAFBAD"))
    }
}

#[derive(Deserialize, Serialize)]
pub struct MetaInput {
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
    pub description: Option<String>,
    pub local: Option<bool>,
}
#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct MetaOutput {
    pub id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
    pub description: Option<String>,
    pub local: Option<bool>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub min_timestamp: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub max_timestamp: Option<OffsetDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct CreateEmissionFactorRequest {
    pub carrier: String,
    pub unit: String,
    pub factor: f64,
    pub source: String,
    pub source_url: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct CreateEmissionFactorResponse {
    pub id: i32,
    // this is optional because requested emission factor might not exist for creation
    pub carrier: Option<String>,
    pub unit: String,
    pub factor: f64,
    pub source: String,
    pub source_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EmissionFactorFilter {
    pub source: Option<String>,
    pub carrier: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct EmissionFactor {
    pub id: i32,
    pub carrier: String,
    pub unit: String,
    pub factor: f64,
    pub source: String,
    pub source_url: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct MetaRows {
    pub values: Vec<MetaOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct ImportConfig {
    pub files: Option<Vec<String>>,
    pub time_column: String,
    pub timeseries: Vec<MetaInput>,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Some(0),
            per_page: Some(1000),
        }
    }
}
impl Pagination {
    pub fn get_page_or_default(&self) -> i32 {
        self.page.unwrap_or(0)
    }

    pub fn get_per_page_or_default(&self) -> i32 {
        self.per_page.unwrap_or(1000)
    }

    pub fn get_offset(&self) -> i32 {
        self.get_page_or_default() * self.get_per_page_or_default()
    }
}
/// `Resampling` is a struct that represents the resampling configuration which is passed as a query parameter
/// to endpoints that return resampled timeseries data.
/// It contains an `interval` field which is a string that specifies the resampling interval.
/// We assume that the resampling method is always taking the mean.
#[derive(Debug, Deserialize)]
pub struct Resampling {
    pub interval: String,
}

/// Provides a default instance of `Resampling`.
/// The default `interval` is "1hour".
impl Default for Resampling {
    fn default() -> Self {
        Self {
            interval: String::from("1hour"),
        }
    }
}

impl Resampling {
    pub fn map_interval(&self) -> std::result::Result<PgInterval, anyhow::Error> {
        let re = Regex::new(r"(\d+)(\w+)").unwrap();
        let caps = re
            .captures(&self.interval)
            .ok_or_else(|| anyhow!("Invalid interval format"))?;
        let num_part = caps.get(1).map_or("", |m| m.as_str()).parse::<i32>()?;
        let unit_part = caps.get(2).map_or("", |m| m.as_str());

        let duration = match unit_part {
            "month" => PgInterval {
                months: num_part,
                microseconds: 0,
                days: 0,
            },
            "hour" => PgInterval {
                months: 0,
                microseconds: num_part as i64 * 60 * 60 * 1000000,
                days: 0,
            },
            "year" => PgInterval {
                months: 12 * num_part,
                microseconds: 0,
                days: 0,
            },
            "day" => PgInterval {
                months: 0,
                microseconds: 0,
                days: num_part,
            },
            "week" => PgInterval {
                months: 0,
                microseconds: 0,
                days: 7 * num_part,
            },
            "min" => PgInterval {
                months: 0,
                microseconds: num_part as i64 * 60 * 1000000,
                days: 0,
            },
            _ => return Err(anyhow!("invalid interval format")),
        };
        Ok(duration)
    }

    pub fn validate_interval(&self) -> bool {
        let pattern = Regex::new(r"^\d+(min|day|week|month|year|hour)$").expect("Invalid interval");
        pattern.is_match(&self.interval)
    }
}

#[derive(Debug, Deserialize)]
pub struct TimestampFilter {
    #[serde(
        with = "time::serde::rfc3339::option",
        default = "TimestampFilter::default_from"
    )]
    pub from: Option<OffsetDateTime>,
    #[serde(
        with = "time::serde::rfc3339::option",
        default = "TimestampFilter::default_to"
    )]
    pub to: Option<OffsetDateTime>,
}

impl TimestampFilter {
    fn default_from() -> Option<OffsetDateTime> {
        Some(OffsetDateTime::UNIX_EPOCH)
    }

    fn default_to() -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc())
    }
}

impl Default for TimestampFilter {
    fn default() -> Self {
        Self {
            from: Self::default_from(),
            to: Some(OffsetDateTime::now_utc()),
        }
    }
}

// intermediate struct to hold results for local consumption of grid electricity
pub struct Consumption {
    pub bucket: Option<OffsetDateTime>,
    pub bucket_consumption: Option<f64>,
    pub consumption_unit: String,
    pub carrier_proportion: Option<f64>,
    pub carrier_name: String,
}

// struct to hold intermediate results for Scope 1 emissions kpi
#[derive(sqlx::FromRow)]
pub struct ProductionWithEmissions {
    pub bucket: Option<OffsetDateTime>,
    pub source_of_production: String,
    pub production_carrier: String,
    pub production: Option<f64>,
    pub scope_one_emissions: Option<f64>,
    pub emission_factor_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KpiResult {
    pub value: f64,
    pub name: String,
    pub unit: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub from_timestamp: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub to_timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmissionsByCarrier {
    #[serde(with = "time::serde::rfc3339::option")]
    pub bucket: Option<OffsetDateTime>,
    pub carrier_name: Option<String>,
    pub value: Option<f64>,
    pub unit: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ConsumptionByCarrier {
    #[serde(with = "time::serde::rfc3339")]
    pub bucket: OffsetDateTime,
    pub carrier_name: String,
    pub value: f64,
    pub unit: String,
    pub local: bool,
}

#[derive(Debug, Deserialize)]
pub struct EmissionFactorSource {
    #[serde(default = "EmissionFactorSource::default_source")]
    pub source: String,
}
impl Default for EmissionFactorSource {
    fn default() -> Self {
        Self {
            source: String::from("IPCC"),
        }
    }
}
impl EmissionFactorSource {
    pub async fn get_source_or_default(&self, pool: &Pool<Postgres>) -> Result<String, ApiError> {
        let source_exists: (bool,) = sqlx::query_as(
            "select exists (select 1 from emission_factor where emission_factor.source = $1)",
        )
        .bind(&self.source)
        .fetch_one(pool)
        .await?;
        if source_exists.0 {
            Ok(self.source.clone())
        } else {
            Ok(EmissionFactorSource::default().source)
        }
    }
    fn default_source() -> String {
        String::from("IPCC")
    }
}

#[test]
fn test_map_interval() {
    let resample = Resampling {
        interval: String::from("1hour"),
    };

    assert_eq!(
        resample.map_interval().unwrap(),
        PgInterval {
            months: 0,
            days: 0,
            microseconds: 60 * 1000000 * 60
        }
    );

    let resample = Resampling {
        interval: String::from("30min"),
    };

    assert_eq!(
        resample.map_interval().unwrap(),
        PgInterval {
            months: 0,
            days: 0,
            microseconds: 30 * 60 * 1000000
        }
    );

    let resample = Resampling {
        interval: String::from("invalid"),
    };

    assert!(resample.map_interval().is_err());
}
