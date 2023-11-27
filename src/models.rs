use anyhow::anyhow;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::postgres::types::PgInterval;
use std::fmt::Formatter;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::error::ApiError;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TimeseriesMeta {
    pub id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Datapoint {
    pub id: i64,
    pub timestamp: OffsetDateTime,
    pub value: f64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
/// TimescaleDB's `time_bucket` function returns a nullable column
pub struct ResampledDatapoint {
    pub timestamp: Option<OffsetDateTime>,
    pub mean_value: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Timeseries {
    pub datapoints: Vec<Datapoint>,
    pub meta: TimeseriesMeta,
}

pub struct ResampledTimeseries {
    pub datapoints: Vec<ResampledDatapoint>,
    pub meta: TimeseriesMeta,
}

#[derive(Serialize, Deserialize)]
pub struct TimeseriesWithoutMetadata {
    pub id: i64,
    pub series_timestamp: OffsetDateTime,
    pub series_value: f64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Intermediate representation for timeseries data from the database
pub struct TimeseriesWithMetadata {
    pub series_id: i64,
    pub series_timestamp: OffsetDateTime,
    pub series_value: f64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub meta_id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
    pub consumption: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateMetadata {
    name: String,
    unit: String,
    carrier: Option<String>,
    consumption: Option<bool>,
}
#[derive(Deserialize, Serialize)]
pub struct TimeseriesNew {
    #[serde(with = "time::serde::rfc3339")]
    pub series_timestamp: OffsetDateTime,
    pub series_value: f64,
    pub identifier: String,
}

#[derive(Serialize, Deserialize)]
pub struct TimeseriesBody<T = Timeseries> {
    pub timeseries: T,
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
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaOutput {
    pub id: i32,
    pub identifier: String,
    pub unit: String,
    pub carrier: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MetaRows {
    pub values: Vec<MetaOutput>,
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
            per_page: Some(1000)
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
        let num_part = caps.get(1).map_or("", |m| m.as_str()).parse::<i64>()?;
        let unit_part = caps.get(2).map_or("", |m| m.as_str());

        let duration = match unit_part {
            "min" => Duration::minutes(num_part),
            "hour" => Duration::hours(num_part),
            "day" => Duration::days(num_part),
            "week" => Duration::weeks(num_part),
            "month" => Duration::weeks(4 * num_part), // Approximation
            "year" => Duration::weeks(52 * num_part), // Approximation
            _ => return Err(anyhow!("invalid interval format")),
        };

        let encoded = PgInterval::try_from(duration).unwrap();
        Ok(encoded)
    }
}
#[test]
fn test_map_interval() {
    let resample = Resampling {
        interval: String::from("1hour"),
    };

    assert_eq!(
        resample.map_interval().unwrap(),
        PgInterval::try_from(Duration::hours(1)).unwrap()
    );

    let resample = Resampling {
        interval: String::from("30min"),
    };

    assert_eq!(
        resample.map_interval().unwrap(),
        PgInterval::try_from(Duration::minutes(30)).unwrap()
    );

    let resample = Resampling {
        interval: String::from("invalid"),
    };

    assert!(resample.map_interval().is_err());
}
