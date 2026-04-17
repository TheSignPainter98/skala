use std::fmt::Display;

use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Encode, Sqlite, Type};

// TODO(kcza): communicate the reactor parameter constraints! E.g. critical temperature,
// ranges of certain values.
#[derive(Debug, quicktype::Quicktype, serde::Deserialize)]
#[serde(tag = "status")]
#[serde(rename_all = "kebab-case")]
#[quicktype(namespace = "server")]
pub enum ReactorState {
    Intact(IntactReactorState),
    Destroyed,
}

#[derive(Debug, quicktype::Quicktype, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub struct IntactReactorState {
    pub mode: ReactorMode,
    pub temperature: f64,
    pub coolant_filled: f64,
    pub heated_coolant_filled: f64,
    pub fuel_filled: f64,
    pub waste_filled: f64,
    pub actual_burn_rate: f64,
    pub target_burn_rate: f64,
    pub damage_percent: f64,
    pub heating_rate: f64,
    pub boil_efficiency: f64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, quicktype::Quicktype, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub enum ReactorMode {
    Inactive,
    Active,
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

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct ReactorId(i64);

impl From<i64> for ReactorId {
    fn from(inner: i64) -> Self {
        Self(inner)
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
