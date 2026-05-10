use std::fmt::Display;

use anyhow::anyhow;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Encode, Sqlite, Type};

use crate::Error;

#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, serde::Serialize)]
#[serde(tag = "integrity")]
#[serde(rename_all = "kebab-case")]
#[quicktype(namespace = "server")]
pub enum ReactorSnapshot {
    Intact(IntactReactorSnapshot),
    Destroyed,
}

#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub struct IntactReactorSnapshot {
    pub mode: ReactorMode,
    pub temperature: f64,
    pub coolant_filled: f64,
    pub heated_coolant_filled: f64,
    pub fuel_filled: f64,
    pub waste_filled: f64,
    pub actual_burn_rate: ActualBurnRate,
    pub target_burn_rate: TargetBurnRate,
    pub max_burn_rate: MaxBurnRate,
    pub damage_percent: f64,
    pub heating_rate: f64,
    pub boil_efficiency: f64,
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, quicktype::Quicktype, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub enum ReactorMode {
    Inactive,
    Active,
}

impl TryFrom<i64> for ReactorMode {
    type Error = Error;

    fn try_from(raw: i64) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Self::Inactive),
            1 => Ok(Self::Active),
            _ => Err(anyhow!("invalid reactor mode {raw}").into()),
        }
    }
}

impl Type<Sqlite> for ReactorMode {
    fn type_info() -> SqliteTypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for ReactorMode {
    fn encode(
        self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        self.encode_by_ref(buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        let value = match self {
            Self::Inactive => 0,
            Self::Active => 1,
        };
        <i64 as Encode<Sqlite>>::encode(value, buf)
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    quicktype::Quicktype,
    schemars::JsonSchema,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TargetBurnRate(i64);

impl From<i64> for TargetBurnRate {
    fn from(rate: i64) -> Self {
        Self(rate)
    }
}

impl Display for TargetBurnRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(rate) = self;
        write!(f, "{rate}mL/s")
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    quicktype::Quicktype,
    schemars::JsonSchema,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct MaxBurnRate(i64);

impl From<i64> for MaxBurnRate {
    fn from(rate: i64) -> Self {
        Self(rate)
    }
}

impl Display for MaxBurnRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(rate) = self;
        write!(f, "{rate}mL/s")
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    quicktype::Quicktype,
    schemars::JsonSchema,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct ActualBurnRate(f64);

impl From<f64> for ActualBurnRate {
    fn from(rate: f64) -> Self {
        Self(rate.round())
    }
}

impl Display for ActualBurnRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(rate) = self;
        write!(f, "{rate}mL/s")
    }
}

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct ReactorId(i64);

impl From<i64> for ReactorId {
    fn from(inner: i64) -> Self {
        Self(inner)
    }
}

impl Display for ReactorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
#[quicktype(namespace = "server")]
pub(crate) struct ReactorName(String);

impl Display for ReactorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(name) = self;
        name.fmt(f)
    }
}
