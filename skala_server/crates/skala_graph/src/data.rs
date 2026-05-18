use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8Path;
use chrono::NaiveDateTime;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow};
use sqlx::{Row, SqlitePool};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetricKey {
    Temperature,
    CoolantFilled,
    HeatedCoolantFilled,
    FuelFilled,
    WasteFilled,
    ActualReactivity,
    TargetReactivity,
    MaxReactivity,
    DamagePercent,
    HeatingRate,
    BoilEfficiency,
    StoredKineticEnergy,
    EnergyProductionRate,
    AdviceNewTargetReactivity,
    ProductionTargetRate,
}

impl MetricKey {
    pub const ALL: [Self; 15] = [
        Self::Temperature,
        Self::CoolantFilled,
        Self::HeatedCoolantFilled,
        Self::FuelFilled,
        Self::WasteFilled,
        Self::ActualReactivity,
        Self::TargetReactivity,
        Self::MaxReactivity,
        Self::DamagePercent,
        Self::HeatingRate,
        Self::BoilEfficiency,
        Self::StoredKineticEnergy,
        Self::EnergyProductionRate,
        Self::AdviceNewTargetReactivity,
        Self::ProductionTargetRate,
    ];

    pub const DEFAULTS: [Self; 4] = [
        Self::DamagePercent,
        Self::EnergyProductionRate,
        Self::AdviceNewTargetReactivity,
        Self::ProductionTargetRate,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Temperature => "Temperature",
            Self::CoolantFilled => "Coolant filled",
            Self::HeatedCoolantFilled => "Heated coolant filled",
            Self::FuelFilled => "Fuel filled",
            Self::WasteFilled => "Waste filled",
            Self::ActualReactivity => "Actual reactivity",
            Self::TargetReactivity => "Target reactivity",
            Self::MaxReactivity => "Max reactivity",
            Self::DamagePercent => "Reactor damage",
            Self::HeatingRate => "Heating rate",
            Self::BoilEfficiency => "Boil efficiency",
            Self::StoredKineticEnergy => "Stored kinetic energy",
            Self::EnergyProductionRate => "Energy production rate",
            Self::AdviceNewTargetReactivity => "Advised reactivity",
            Self::ProductionTargetRate => "Production target",
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            Self::Temperature => "K",
            Self::CoolantFilled => "mB",
            Self::HeatedCoolantFilled => "mB",
            Self::FuelFilled => "mB",
            Self::WasteFilled => "mB",
            Self::ActualReactivity => "mB/t",
            Self::TargetReactivity => "mB/t",
            Self::MaxReactivity => "mB/t",
            Self::DamagePercent => "%",
            Self::HeatingRate => "K/t",
            Self::BoilEfficiency => "%",
            Self::StoredKineticEnergy => "J",
            Self::EnergyProductionRate => "J/t",
            Self::AdviceNewTargetReactivity => "mB/t",
            Self::ProductionTargetRate => "J/t",
        }
    }

    pub fn column_alias(self) -> &'static str {
        match self {
            Self::Temperature => "temperature",
            Self::CoolantFilled => "coolant_filled",
            Self::HeatedCoolantFilled => "heated_coolant_filled",
            Self::FuelFilled => "fuel_filled",
            Self::WasteFilled => "waste_filled",
            Self::ActualReactivity => "actual_burn_rate",
            Self::TargetReactivity => "target_burn_rate",
            Self::MaxReactivity => "max_burn_rate",
            Self::DamagePercent => "damage_percent",
            Self::HeatingRate => "heating_rate",
            Self::BoilEfficiency => "boil_efficiency",
            Self::StoredKineticEnergy => "stored_kinetic_energy",
            Self::EnergyProductionRate => "energy_production_rate",
            Self::AdviceNewTargetReactivity => "advice_new_target_burn_rate",
            Self::ProductionTargetRate => "production_target_rate",
        }
    }
}

impl Display for MetricKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.title())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactorSummary {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataPoint {
    pub ingame_time: NaiveDateTime,
    pub raw_values: HashMap<MetricKey, Option<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricSeries {
    pub key: MetricKey,
    pub points: Vec<(NaiveDateTime, f64)>,
    pub raw_min: f64,
    pub raw_max: f64,
    pub is_constant: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReactorData {
    pub reactor: ReactorSummary,
    pub points: Vec<DataPoint>,
    pub available_metrics: BTreeSet<MetricKey>,
}

pub async fn open_database(path: &Utf8Path) -> Result<SqlitePool> {
    let connect_options =
        SqliteConnectOptions::from_str(&format!("sqlite://{path}?"))?.read_only(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await
        .with_context(|| format!("failed to open SQLite database at {path}"))?;
    Ok(pool)
}

pub async fn load_reactors(connection: &SqlitePool) -> Result<Vec<ReactorSummary>> {
    let rows = sqlx::query(
        "
        SELECT reactor.id, reactor.name
        FROM reactor
        JOIN event ON event.reactor_id = reactor.id
        GROUP BY reactor.id, reactor.name
        ORDER BY reactor.name
        ",
    )
    .fetch_all(connection)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ReactorSummary {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
            })
        })
        .collect()
}

pub fn select_reactor(reactors: &[ReactorSummary], name: Option<&str>) -> Result<usize> {
    if reactors.is_empty() {
        bail!("the database does not contain any reactors with events");
    }

    match name {
        Some(name) => reactors
            .iter()
            .position(|reactor| reactor.name == name)
            .ok_or_else(|| {
                let known = reactors
                    .iter()
                    .map(|reactor| reactor.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow!("reactor `{name}` was not found; known reactors: {known}")
            }),
        None => Ok(0),
    }
}

pub async fn load_reactor_data(
    connection: &SqlitePool,
    reactor: ReactorSummary,
) -> Result<ReactorData> {
    let rows = sqlx::query(
        "
        SELECT
            event.ingame_timestamp,
            snapshot.temperature,
            snapshot.coolant_filled,
            snapshot.heated_coolant_filled,
            snapshot.fuel_filled,
            snapshot.waste_filled,
            snapshot.actual_burn_rate,
            snapshot.target_burn_rate,
            snapshot.max_burn_rate,
            snapshot.damage_percent,
            snapshot.heating_rate,
            snapshot.boil_efficiency,
            snapshot.stored_kinetic_energy,
            snapshot.energy_production_rate,
            advice.new_target_burn_rate,
            production_target.rate
        FROM event
        LEFT JOIN snapshot ON snapshot.event_id = event.id
        LEFT JOIN advice ON advice.event_id = event.id
        LEFT JOIN production_target ON production_target.event_id = event.id
        WHERE event.reactor_id = ?
        ORDER BY event.irl_timestamp ASC, event.id ASC
        ",
    )
    .bind(reactor.id)
    .fetch_all(connection)
    .await?;

    let mut points = Vec::new();
    let mut available_metrics = BTreeSet::new();

    for row in &rows {
        let point = row_to_point(row)?;
        for (key, value) in &point.raw_values {
            if value.is_some() {
                available_metrics.insert(*key);
            }
        }
        points.push(point);
    }

    Ok(ReactorData {
        reactor,
        points,
        available_metrics,
    })
}

pub fn build_series(data: &ReactorData, metric: MetricKey) -> Option<MetricSeries> {
    let values = data
        .points
        .iter()
        .filter_map(|point| {
            point
                .raw_values
                .get(&metric)
                .and_then(|value| value.map(|value| (point.ingame_time, value)))
        })
        .collect::<Vec<_>>();

    if values.is_empty() {
        return None;
    }

    let raw_min = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::min)
        .expect("values is non-empty");
    let raw_max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .expect("values is non-empty");
    let is_constant = (raw_max - raw_min).abs() < f64::EPSILON;

    Some(MetricSeries {
        key: metric,
        points: values,
        raw_min,
        raw_max,
        is_constant,
    })
}

fn row_to_point(row: &SqliteRow) -> Result<DataPoint> {
    let timestamp_text: String = row.try_get(0)?;
    let ingame_time = NaiveDateTime::parse_from_str(&timestamp_text, "%Y-%m-%dT%H:%M:%S")
        .with_context(|| format!("invalid ingame_timestamp `{timestamp_text}`"))?;

    let raw_values = HashMap::from([
        (MetricKey::Temperature, numeric_cell(row, 1)?),
        (MetricKey::CoolantFilled, numeric_cell(row, 2)?),
        (MetricKey::HeatedCoolantFilled, numeric_cell(row, 3)?),
        (MetricKey::FuelFilled, numeric_cell(row, 4)?),
        (MetricKey::WasteFilled, numeric_cell(row, 5)?),
        (MetricKey::ActualReactivity, numeric_cell(row, 6)?),
        (MetricKey::TargetReactivity, numeric_cell(row, 7)?),
        (MetricKey::MaxReactivity, numeric_cell(row, 8)?),
        (MetricKey::DamagePercent, numeric_cell(row, 9)?),
        (MetricKey::HeatingRate, numeric_cell(row, 10)?),
        (MetricKey::BoilEfficiency, numeric_cell(row, 11)?),
        (MetricKey::StoredKineticEnergy, numeric_cell(row, 12)?),
        (MetricKey::EnergyProductionRate, numeric_cell(row, 13)?),
        (MetricKey::AdviceNewTargetReactivity, numeric_cell(row, 14)?),
        (MetricKey::ProductionTargetRate, numeric_cell(row, 15)?),
    ]);

    Ok(DataPoint {
        ingame_time,
        raw_values,
    })
}

fn numeric_cell(row: &SqliteRow, index: usize) -> Result<Option<f64>> {
    match row.try_get::<Option<f64>, _>(index) {
        Ok(value) => Ok(value),
        Err(float_error) => row
            .try_get::<Option<i64>, _>(index)
            .map(|value| value.map(|value| value as f64))
            .map_err(|_| float_error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Executor;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_connection() -> SqlitePool {
        let connection = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory db");
        sqlx::raw_sql(
            "
                CREATE TABLE event (
                    id INTEGER PRIMARY KEY,
                    reactor_id INTEGER NOT NULL,
                    irl_timestamp INTEGER NOT NULL,
                    ingame_timestamp TEXT NOT NULL
                ) STRICT;
                CREATE TABLE reactor (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE
                ) STRICT;
                CREATE TABLE snapshot (
                    event_id INTEGER PRIMARY KEY,
                    temperature REAL NULL,
                    coolant_filled REAL NULL,
                    heated_coolant_filled REAL NULL,
                    fuel_filled REAL NULL,
                    waste_filled REAL NULL,
                    actual_burn_rate REAL NULL,
                    target_burn_rate INTEGER NULL,
                    max_burn_rate INTEGER NULL,
                    damage_percent REAL NULL,
                    heating_rate REAL NULL,
                    boil_efficiency REAL NULL,
                    stored_kinetic_energy REAL NULL,
                    energy_production_rate REAL NULL
                ) STRICT;
                CREATE TABLE advice (
                    event_id INTEGER PRIMARY KEY,
                    action INTEGER NOT NULL,
                    pretty_action TEXT,
                    new_target_burn_rate INTEGER NULL,
                    reasoning TEXT NOT NULL
                ) STRICT;
                CREATE TABLE production_target (
                    event_id INTEGER PRIMARY KEY,
                    rate REAL NOT NULL
                ) STRICT;
                ",
        )
        .execute(&connection)
        .await
        .expect("schema");
        connection
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_series_handles_sparse_values() {
        let connection = setup_connection().await;
        connection
            .execute("INSERT INTO reactor (id, name) VALUES (1, 'reactor_a')")
            .await
            .expect("insert reactor");
        connection
            .execute(
                "INSERT INTO event (id, reactor_id, irl_timestamp, ingame_timestamp) VALUES (1, 1, 100, '2026-05-10T22:45:21')",
            )
            .await
            .expect("insert event");
        connection
            .execute(
                "INSERT INTO event (id, reactor_id, irl_timestamp, ingame_timestamp) VALUES (2, 1, 200, '2026-05-10T22:45:31')",
            )
            .await
            .expect("insert event");
        connection
            .execute(
                "INSERT INTO snapshot (event_id, temperature, actual_burn_rate, target_burn_rate, energy_production_rate) VALUES (1, 5.0, 1.0, 10, 100.0)",
            )
            .await
            .expect("insert snapshot");
        connection
            .execute(
                "INSERT INTO snapshot (event_id, temperature, actual_burn_rate, target_burn_rate, energy_production_rate) VALUES (2, 15.0, 2.0, 20, 200.0)",
            )
            .await
            .expect("insert snapshot");
        connection
            .execute(
                "INSERT INTO advice (event_id, action, pretty_action, new_target_burn_rate, reasoning) VALUES (1, 2, 'set-target-reactivity', NULL, 'n/a')",
            )
            .await
            .expect("insert advice");
        connection
            .execute(
                "INSERT INTO advice (event_id, action, pretty_action, new_target_burn_rate, reasoning) VALUES (2, 2, 'set-target-reactivity', 30, 'n/a')",
            )
            .await
            .expect("insert advice");

        let data = load_reactor_data(
            &connection,
            ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            },
        )
        .await
        .expect("load reactor data");
        let sparse = build_series(&data, MetricKey::AdviceNewTargetReactivity).expect("series");

        assert_eq!(sparse.points.len(), 1);
        assert_eq!(sparse.raw_min, 30.0);
        assert_eq!(sparse.raw_max, 30.0);
        assert!(sparse.is_constant);
    }

    #[test]
    fn normalisation_handles_constant_series() {
        let data = ReactorData {
            reactor: ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            },
            points: vec![
                DataPoint {
                    ingame_time: NaiveDateTime::parse_from_str(
                        "2026-05-10T22:45:21",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .expect("timestamp"),
                    raw_values: HashMap::from([(MetricKey::TargetReactivity, Some(10.0))]),
                },
                DataPoint {
                    ingame_time: NaiveDateTime::parse_from_str(
                        "2026-05-10T22:45:31",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .expect("timestamp"),
                    raw_values: HashMap::from([(MetricKey::TargetReactivity, Some(10.0))]),
                },
            ],
            available_metrics: BTreeSet::from([MetricKey::TargetReactivity]),
        };

        let series = build_series(&data, MetricKey::TargetReactivity).expect("series");
        assert_eq!(series.points[0].1, 10.0);
        assert_eq!(series.points[1].1, 10.0);
    }
}
