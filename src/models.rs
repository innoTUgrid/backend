use crate::error::ApiError;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

        // By providing our own `Visitor` impl, we can access the string data without copying.
        //
        // We could deserialize a borrowed `&str` directly but certain deserialization modes
        // of `serde_json` don't support that, so we'd be forced to always deserialize `String`.
        //
        // `serde_with` has a helper for this but it can be a bit overkill to bring in
        // just for one type: https://docs.rs/serde_with/latest/serde_with/#displayfromstr
        //
        // We'd still need to implement `Display` and `FromStr`, but those are much simpler
        // to work with.
        //
        // However, I also wanted to demonstrate that it was possible to do this with Serde alone.
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

#[derive(Serialize)]
pub struct TimeseriesFlat {
    pub id: i64,
    pub series_timestamp: OffsetDateTime,
    pub series_value: f64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct Timeseries {
    pub id: i64,
    pub series_timestamp: Timestamptz,
    pub series_value: f64,
    pub created_at: Timestamptz,
    pub updated_at: Timestamptz,
    pub meta: TimeseriesMeta,
}

pub struct TimeseriesFromQuery {
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

impl TimeseriesFromQuery {
    pub fn into_timeseries(self) -> Timeseries {
        Timeseries {
            id: self.series_id,
            series_timestamp: Timestamptz(self.series_timestamp),
            series_value: self.series_value,
            created_at: Timestamptz(self.created_at),
            updated_at: Timestamptz(self.updated_at),
            meta: TimeseriesMeta {
                id: self.meta_id,
                identifier: self.identifier,
                unit: self.unit,
                carrier: self.carrier,
                consumption: self.consumption,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CreateMetadata {
    name: String,
    unit: String,
    carrier: Option<String>,
    consumption: Option<bool>,
}

struct GetMetadata {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Deserialize)]
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
#[derive(Serialize)]
pub struct TimeseriesResponse {
    pub data: Vec<Timeseries>,
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

#[derive(Debug, Clone, Serialize)]
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
