use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDateTime;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, params};

const REQUIRED_TABLES: &[&str] = &[
    "event",
    "reactor",
    "snapshot",
    "advice",
    "production_target",
];
const REQUIRED_EVENT_COLUMNS: &[&str] = &["id", "reactor_id", "irl_timestamp", "ingame_timestamp"];
const REQUIRED_SNAPSHOT_COLUMNS: &[&str] = &[
    "event_id",
    "temperature",
    "coolant_filled",
    "heated_coolant_filled",
    "fuel_filled",
    "waste_filled",
    "actual_burn_rate",
    "target_burn_rate",
    "max_burn_rate",
    "damage_percent",
    "heating_rate",
    "boil_efficiency",
    "stored_kinetic_energy",
    "energy_production_rate",
];
const REQUIRED_ADVICE_COLUMNS: &[&str] = &["event_id", "new_target_burn_rate"];
const REQUIRED_PRODUCTION_TARGET_COLUMNS: &[&str] = &["event_id", "rate"];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetricKey {
    Temperature,
    CoolantFilled,
    HeatedCoolantFilled,
    FuelFilled,
    WasteFilled,
    ActualBurnRate,
    TargetBurnRate,
    MaxBurnRate,
    DamagePercent,
    HeatingRate,
    BoilEfficiency,
    StoredKineticEnergy,
    EnergyProductionRate,
    AdviceNewTargetBurnRate,
    ProductionTargetRate,
}

impl MetricKey {
    pub const ALL: [Self; 15] = [
        Self::Temperature,
        Self::CoolantFilled,
        Self::HeatedCoolantFilled,
        Self::FuelFilled,
        Self::WasteFilled,
        Self::ActualBurnRate,
        Self::TargetBurnRate,
        Self::MaxBurnRate,
        Self::DamagePercent,
        Self::HeatingRate,
        Self::BoilEfficiency,
        Self::StoredKineticEnergy,
        Self::EnergyProductionRate,
        Self::AdviceNewTargetBurnRate,
        Self::ProductionTargetRate,
    ];

    pub const DEFAULTS: [Self; 4] = [
        Self::DamagePercent,
        Self::EnergyProductionRate,
        Self::AdviceNewTargetBurnRate,
        Self::ProductionTargetRate,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Temperature => "Temperature",
            Self::CoolantFilled => "Coolant filled",
            Self::HeatedCoolantFilled => "Heated coolant filled",
            Self::FuelFilled => "Fuel filled",
            Self::WasteFilled => "Waste filled",
            Self::ActualBurnRate => "Actual burn rate",
            Self::TargetBurnRate => "Target burn rate",
            Self::MaxBurnRate => "Max burn rate",
            Self::DamagePercent => "Reactor damage",
            Self::HeatingRate => "Heating rate",
            Self::BoilEfficiency => "Boil efficiency",
            Self::StoredKineticEnergy => "Stored kinetic energy",
            Self::EnergyProductionRate => "Energy production rate",
            Self::AdviceNewTargetBurnRate => "Advised Burn Rate",
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
            Self::ActualBurnRate => "mB/t",
            Self::TargetBurnRate => "mB/t",
            Self::MaxBurnRate => "mB/t",
            Self::DamagePercent => "%",
            Self::HeatingRate => "K/t",
            Self::BoilEfficiency => "%",
            Self::StoredKineticEnergy => "J",
            Self::EnergyProductionRate => "J/t",
            Self::AdviceNewTargetBurnRate => "mB/t",
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
            Self::ActualBurnRate => "actual_burn_rate",
            Self::TargetBurnRate => "target_burn_rate",
            Self::MaxBurnRate => "max_burn_rate",
            Self::DamagePercent => "damage_percent",
            Self::HeatingRate => "heating_rate",
            Self::BoilEfficiency => "boil_efficiency",
            Self::StoredKineticEnergy => "stored_kinetic_energy",
            Self::EnergyProductionRate => "energy_production_rate",
            Self::AdviceNewTargetBurnRate => "advice_new_target_burn_rate",
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

pub fn open_database(path: &Path) -> Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("failed to open SQLite database at {}", path.display()))
}

pub fn validate_schema(connection: &Connection) -> Result<()> {
    for table in REQUIRED_TABLES {
        if !table_exists(connection, table)? {
            bail!("required table `{table}` is missing");
        }
    }

    ensure_columns(connection, "event", REQUIRED_EVENT_COLUMNS)?;
    ensure_columns(connection, "snapshot", REQUIRED_SNAPSHOT_COLUMNS)?;
    ensure_columns(connection, "advice", REQUIRED_ADVICE_COLUMNS)?;
    ensure_columns(
        connection,
        "production_target",
        REQUIRED_PRODUCTION_TARGET_COLUMNS,
    )?;

    Ok(())
}

pub fn load_reactors(connection: &Connection) -> Result<Vec<ReactorSummary>> {
    let mut statement = connection.prepare(
        "
        SELECT reactor.id, reactor.name
        FROM reactor
        JOIN event ON event.reactor_id = reactor.id
        GROUP BY reactor.id, reactor.name
        ORDER BY reactor.name
        ",
    )?;

    let reactors = statement
        .query_map([], |row| {
            Ok(ReactorSummary {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(reactors)
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

pub fn load_reactor_data(connection: &Connection, reactor: ReactorSummary) -> Result<ReactorData> {
    let mut statement = connection.prepare(
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
    )?;

    let mut rows = statement.query(params![reactor.id])?;
    let mut points = Vec::new();
    let mut available_metrics = BTreeSet::new();

    while let Some(row) = rows.next()? {
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

fn row_to_point(row: &Row<'_>) -> Result<DataPoint> {
    let timestamp_text: String = row.get(0)?;
    let ingame_time = NaiveDateTime::parse_from_str(&timestamp_text, "%Y-%m-%dT%H:%M:%S")
        .with_context(|| format!("invalid ingame_timestamp `{timestamp_text}`"))?;

    let raw_values = HashMap::from([
        (MetricKey::Temperature, row.get(1)?),
        (MetricKey::CoolantFilled, row.get(2)?),
        (MetricKey::HeatedCoolantFilled, row.get(3)?),
        (MetricKey::FuelFilled, row.get(4)?),
        (MetricKey::WasteFilled, row.get(5)?),
        (MetricKey::ActualBurnRate, row.get(6)?),
        (MetricKey::TargetBurnRate, row.get(7)?),
        (MetricKey::MaxBurnRate, row.get(8)?),
        (MetricKey::DamagePercent, row.get(9)?),
        (MetricKey::HeatingRate, row.get(10)?),
        (MetricKey::BoilEfficiency, row.get(11)?),
        (MetricKey::StoredKineticEnergy, row.get(12)?),
        (MetricKey::EnergyProductionRate, row.get(13)?),
        (MetricKey::AdviceNewTargetBurnRate, row.get(14)?),
        (MetricKey::ProductionTargetRate, row.get(15)?),
    ]);

    Ok(DataPoint {
        ingame_time,
        raw_values,
    })
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool> {
    Ok(connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1",
            params![table],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some())
}

fn ensure_columns(connection: &Connection, table: &str, required_columns: &[&str]) -> Result<()> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut statement = connection.prepare(&pragma)?;
    let existing = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<BTreeSet<_>>>()?;

    for column in required_columns {
        if !existing.contains(*column) {
            bail!("required column `{table}.{column}` is missing");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory db");
        connection
            .execute_batch(
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
            .expect("schema");
        connection
    }

    #[test]
    fn schema_validation_accepts_expected_schema() {
        let connection = setup_connection();
        validate_schema(&connection).expect("schema should validate");
    }

    #[test]
    fn schema_validation_rejects_missing_table() {
        let connection = Connection::open_in_memory().expect("in-memory db");
        let error = validate_schema(&connection).expect_err("schema should fail");
        assert!(
            error
                .to_string()
                .contains("required table `event` is missing")
        );
    }

    #[test]
    fn schema_validation_rejects_missing_event_column() {
        let connection = Connection::open_in_memory().expect("in-memory db");
        connection
            .execute_batch(
                "
                CREATE TABLE event (
                    id INTEGER PRIMARY KEY,
                    reactor_id INTEGER NOT NULL,
                    irl_timestamp INTEGER NOT NULL
                ) STRICT;
                CREATE TABLE reactor (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE) STRICT;
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
                CREATE TABLE production_target (event_id INTEGER PRIMARY KEY, rate REAL NOT NULL) STRICT;
                ",
            )
            .expect("schema");

        let error = validate_schema(&connection).expect_err("schema should fail");
        assert!(
            error
                .to_string()
                .contains("required column `event.ingame_timestamp` is missing")
        );
    }

    #[test]
    fn load_series_handles_sparse_values() {
        let connection = setup_connection();
        connection
            .execute("INSERT INTO reactor (id, name) VALUES (1, 'reactor_a')", [])
            .expect("insert reactor");
        connection
            .execute(
                "INSERT INTO event (id, reactor_id, irl_timestamp, ingame_timestamp) VALUES (1, 1, 100, '2026-05-10T22:45:21')",
                [],
            )
            .expect("insert event");
        connection
            .execute(
                "INSERT INTO event (id, reactor_id, irl_timestamp, ingame_timestamp) VALUES (2, 1, 200, '2026-05-10T22:45:31')",
                [],
            )
            .expect("insert event");
        connection
            .execute(
                "INSERT INTO snapshot (event_id, temperature, actual_burn_rate, target_burn_rate, energy_production_rate) VALUES (1, 5.0, 1.0, 10, 100.0)",
                [],
            )
            .expect("insert snapshot");
        connection
            .execute(
                "INSERT INTO snapshot (event_id, temperature, actual_burn_rate, target_burn_rate, energy_production_rate) VALUES (2, 15.0, 2.0, 20, 200.0)",
                [],
            )
            .expect("insert snapshot");
        connection
            .execute(
                "INSERT INTO advice (event_id, action, pretty_action, new_target_burn_rate, reasoning) VALUES (1, 2, 'set-target-burn-rate', NULL, 'n/a')",
                [],
            )
            .expect("insert advice");
        connection
            .execute(
                "INSERT INTO advice (event_id, action, pretty_action, new_target_burn_rate, reasoning) VALUES (2, 2, 'set-target-burn-rate', 30, 'n/a')",
                [],
            )
            .expect("insert advice");

        let data = load_reactor_data(
            &connection,
            ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            },
        )
        .expect("load reactor data");
        let sparse = build_series(&data, MetricKey::AdviceNewTargetBurnRate).expect("series");

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
                    raw_values: HashMap::from([(MetricKey::TargetBurnRate, Some(10.0))]),
                },
                DataPoint {
                    ingame_time: NaiveDateTime::parse_from_str(
                        "2026-05-10T22:45:31",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .expect("timestamp"),
                    raw_values: HashMap::from([(MetricKey::TargetBurnRate, Some(10.0))]),
                },
            ],
            available_metrics: BTreeSet::from([MetricKey::TargetBurnRate]),
        };

        let series = build_series(&data, MetricKey::TargetBurnRate).expect("series");
        assert_eq!(series.points[0].1, 10.0);
        assert_eq!(series.points[1].1, 10.0);
    }
}
