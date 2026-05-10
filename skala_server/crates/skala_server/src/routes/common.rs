use std::fmt::Display;

use sqlx::{SqliteTransaction, query, query_as};

use crate::Result;
use crate::components::reactor::{ReactorId, ReactorName};
use crate::time::{IngameDateTime, IrlDateTime};

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct EventId(i64);

impl From<i64> for EventId {
    fn from(inner: i64) -> Self {
        Self(inner)
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

pub(crate) async fn get_reactor_id(
    txn: &mut SqliteTransaction<'_>,
    name: &ReactorName,
) -> Result<ReactorId> {
    struct Row {
        id: i64,
    }
    let id_query = query_as!(
        Row,
        "
            SELECT id
            FROM reactor
            WHERE name = ?
        ",
        name,
    );
    let id = id_query.fetch_optional(&mut **txn).await?;
    if let Some(Row { id }) = id {
        return Ok(ReactorId::from(id));
    }

    let name_insertion_query = query!(
        "
            INSERT INTO reactor (name)
            VALUES (?)
        ",
        name
    );
    let info = name_insertion_query.execute(&mut **txn).await?;
    Ok(ReactorId::from(info.last_insert_rowid()))
}

pub(crate) async fn register_event(
    txn: &mut SqliteTransaction<'_>,
    reactor_id: ReactorId,
    irl_timestamp: IrlDateTime,
    ingame_timestamp: IngameDateTime,
) -> Result<EventId> {
    let irl_timestamp = irl_timestamp.unix_timestamp();
    let event_insertion_query = query!(
        "
            INSERT INTO event (reactor_id, irl_timestamp, ingame_timestamp)
            VALUES (?, ?, ?)
        ",
        reactor_id,
        irl_timestamp,
        ingame_timestamp,
    );
    let id = event_insertion_query
        .execute(&mut **txn)
        .await?
        .last_insert_rowid();
    Ok(EventId(id))
}
