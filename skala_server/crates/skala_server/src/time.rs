use std::fmt::Display;

use time::UtcDateTime;

#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[quicktype(namespace = "server")]
pub struct IngameDateTime(String);

impl IngameDateTime {
    pub fn into_inner(self) -> String {
        let Self(inner) = self;
        inner
    }
}

impl From<String> for IngameDateTime {
    fn from(inner: String) -> Self {
        Self(inner)
    }
}

impl Display for IngameDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct IrlDateTime(UtcDateTime);

impl IrlDateTime {
    pub fn now() -> Self {
        Self(UtcDateTime::now())
    }

    pub fn unix_timestamp(&self) -> i64 {
        let Self(inner) = self;
        inner.unix_timestamp()
    }
}
