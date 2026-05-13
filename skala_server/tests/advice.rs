mod common;

use anyhow::anyhow;
use serde_json::{Value, json};
use skala_server::advisor::{Advice, AdvisedAction, PastEvent, Snapshot, SystemKnowledge};
use skala_server::{
    ActualBurnRate, IntactReactorSnapshot, IntactTurbineSnapshot, MaxBurnRate, ReactorMode,
    ReactorSnapshot, TargetBurnRate, TurbineSnapshot,
};
use sqlx::{Row, SqlitePool};

use crate::common::MockAdvisor;

type PastEventsCheck = Box<dyn Fn(Vec<&PastEvent>) + Send>;
type TargetBurnRateCheck = Box<dyn Fn(f64) + Send>;
type SystemKnowledgeCheck = Box<dyn Fn(Option<&SystemKnowledge>) + Send>;

#[sqlx::test(migrations = "./migrations")]
async fn test_destroyed_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "target_energy_production_rate": 1000.0,
            "reactor_state": {
                "integrity": "destroyed"
            },
            "turbine_state": {
                "integrity": "destroyed",
            },
            "timestamp": "2026-04-15T00:00:00"
        }))
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
        }))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_inactive_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const TIMESTAMP: &str = "2026-04-15T00:00:00";
    const TARGET_ENERGY_PRODUCTION_RATE: f64 = 1234.5;

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "target_energy_production_rate": TARGET_ENERGY_PRODUCTION_RATE,
            "reactor_state": {
                "integrity": "intact",
                "mode": "inactive",
                "temperature": 111.0,
                "coolant_filled_percent": 222.0,
                "heated_coolant_filled_percent": 333.0,
                "fuel_filled_percent": 444.0,
                "waste_filled_percent": 555.0,
                "actual_burn_rate": 666.0,
                "target_burn_rate": 777,
                "max_burn_rate": 123,
                "damage_percent": 888.0,
                "heating_rate": 999.0,
                "boil_efficiency_percent": 1234.0,
            },
            "turbine_state": {
                "integrity": "intact",
                "stored_kinetic_energy": 789.0,
                "energy_production_rate": 456.0,
            },
            "timestamp": TIMESTAMP,
        }))
        .advice(Advice {
            action: AdvisedAction::NoAction,
            reasoning: "all good".into(),
            system_knowledge: SystemKnowledge::from("Inactive reactor remained stable.".to_owned()),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "all good",
                "system_knowledge": "Inactive reactor remained stable.",
            },
        }))
        .check_past_events(|past_events| {
            let past_event = match past_events.as_slice() {
                [] => panic!("too few snapshots"),
                [past_event] => past_event.to_owned(),
                _ => panic!("too many snapshots"),
            };
            let snapshot = match past_event {
                PastEvent::Snapshot(snapshot) => snapshot,
                _ => panic!("incorrect past event"),
            };
            let Snapshot {
                timestamp,
                reactor,
                turbine,
            } = snapshot;
            assert_eq!(timestamp.clone().into_inner(), TIMESTAMP);

            let intact_reactor_snapshot = match reactor {
                ReactorSnapshot::Intact(intact_reactor_state) => intact_reactor_state,
                _ => panic!("unexpected reactor state"),
            };
            let IntactReactorSnapshot {
                mode,
                temperature,
                coolant_filled_percent,
                heated_coolant_filled_percent,
                fuel_filled_percent,
                waste_filled_percent,
                actual_burn_rate,
                target_burn_rate,
                max_burn_rate,
                damage_percent,
                heating_rate,
                boil_efficiency_percent,
            } = intact_reactor_snapshot;
            assert!(matches!(*mode, ReactorMode::Inactive));
            assert_eq!(*temperature, 111.0);
            assert_eq!(*coolant_filled_percent, 222.0.into());
            assert_eq!(*heated_coolant_filled_percent, 333.0.into());
            assert_eq!(*fuel_filled_percent, 444.0.into());
            assert_eq!(*waste_filled_percent, 555.0.into());
            assert_eq!(*actual_burn_rate, ActualBurnRate::from(666.0));
            assert_eq!(*target_burn_rate, TargetBurnRate::from(777));
            assert_eq!(*max_burn_rate, MaxBurnRate::from(123));
            assert_eq!(*damage_percent, 888.0.into());
            assert_eq!(*heating_rate, 999.0);
            assert_eq!(*boil_efficiency_percent, 1234.0.into());

            let intact_turbine_snapshot = match turbine {
                TurbineSnapshot::Intact(intact_turbine_snapshot) => intact_turbine_snapshot,
                _ => panic!("unexpected reactor state"),
            };
            let IntactTurbineSnapshot {
                stored_kinetic_energy,
                energy_production_rate,
            } = intact_turbine_snapshot;
            assert_eq!(*stored_kinetic_energy, 789.0);
            assert_eq!(*energy_production_rate, 456.0);
        })
        .check_system_knowledge(|knowledge| assert!(knowledge.is_none()))
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_active_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const TARGET_ENERGY_PRODUCTION_RATE: f64 = 1234.5;

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "target_energy_production_rate": TARGET_ENERGY_PRODUCTION_RATE,
            "reactor_state": {
                "integrity": "intact",
                "mode": "active",
                "temperature": 0.0,
                "coolant_filled_percent": 0.0,
                "heated_coolant_filled_percent": 0.0,
                "fuel_filled_percent": 0.0,
                "waste_filled_percent": 0.0,
                "actual_burn_rate": 0.0,
                "target_burn_rate": 0,
                "max_burn_rate": 1000,
                "damage_percent": 0.0,
                "heating_rate": 0.0,
                "boil_efficiency_percent": 0.0,
            },
            "turbine_state": {
                "integrity": "intact",
                "stored_kinetic_energy": 789.0,
                "energy_production_rate": 456.0,
            },
            "timestamp": "2026-04-15T00:00:00"
        }))
        .advice(Advice {
            action: AdvisedAction::SetTargetBurnRate {
                new_target_burn_rate: 1000.into(),
            },
            reasoning: "let's see what happens".into(),
            system_knowledge: SystemKnowledge::from(
                "Initial active response requested more burn.".to_owned(),
            ),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "set-target-burn-rate",
                    "new_target_burn_rate": 1000,
                },
                "reasoning": "let's see what happens",
                "system_knowledge": "Initial active response requested more burn.",
            },
        }))
        .check_past_events(|events| {
            assert!(matches!(
                events.as_slice(),
                [PastEvent::Snapshot(Snapshot {
                    reactor: ReactorSnapshot::Intact(IntactReactorSnapshot {
                        mode: ReactorMode::Active,
                        ..
                    }),
                    ..
                })],
            ));
        })
        .check_system_knowledge(|knowledge| assert!(knowledge.is_none()))
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_knowledge_retention(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const INITIAL_SYSTEM_KNOWLEDGE: &str =
        "A 100 mL/s burn-rate increase lifted output after one snapshot.";
    const SYSTEM_KNOWLEDGE: &str = "Holding the new burn rate kept output stable.";

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:00"))
        .advice(Advice {
            action: AdvisedAction::NoAction,
            reasoning: "collect baseline".into(),
            system_knowledge: SystemKnowledge::from(INITIAL_SYSTEM_KNOWLEDGE.to_owned()),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "collect baseline",
                "system_knowledge": INITIAL_SYSTEM_KNOWLEDGE,
            },
        }))
        .check_system_knowledge(|knowledge| assert!(knowledge.is_none()))
        .run(db_pool.clone())
        .await;

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:01"))
        .advice(Advice {
            action: AdvisedAction::NoAction,
            reasoning: "hold steady".into(),
            system_knowledge: SystemKnowledge::from(SYSTEM_KNOWLEDGE.to_owned()),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "hold steady",
                "system_knowledge": SYSTEM_KNOWLEDGE,
            },
        }))
        .check_system_knowledge(|knowledge| {
            let knowledge = knowledge.cloned().map(String::from);
            assert_eq!(knowledge.as_deref(), Some(INITIAL_SYSTEM_KNOWLEDGE));
        })
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_set_target_returns_plain_text(db_pool: SqlitePool) {
    let test_server = common::setup(db_pool, default_advisor());

    let resp = test_server
        .get("/set-target?reactor_name=pop&rate=1500.5")
        .await;

    resp.assert_status_ok();
    assert_eq!(
        "Target energy production rate for reactor pop set to 1500.5",
        resp.text()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_success_response_is_not_logged_as_error(db_pool: SqlitePool) {
    let logs = common::recorded_logs();
    let test_server = common::setup(db_pool, default_advisor());

    let resp = test_server
        .get("/set-target?reactor_name=pop&rate=1500.5")
        .await;

    resp.assert_status_ok();
    assert!(!logs.contains("HTTP 200 OK response for GET /set-target"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_internal_server_error_response_is_logged(db_pool: SqlitePool) {
    const ERROR_RESPONSE: &str = "unique middleware error response";

    let logs = common::recorded_logs();
    let advisor = MockAdvisor::new(|_, _, _| Err(anyhow!(ERROR_RESPONSE).into()));
    let test_server = common::setup(db_pool, advisor);

    let resp = test_server
        .post("/advice")
        .json(&active_request("pop", "2026-04-15T00:00:00"))
        .await;

    resp.assert_status_internal_server_error();
    assert_eq!(ERROR_RESPONSE, resp.text());
    assert!(logs.contains("HTTP 500 Internal Server Error response for POST /advice"));
    assert!(logs.contains(ERROR_RESPONSE));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_not_found_response_is_logged(db_pool: SqlitePool) {
    const PATH: &str = "/unique-middleware-not-found-route";

    let logs = common::recorded_logs();
    let test_server = common::setup(db_pool, default_advisor());

    let resp = test_server.get(PATH).await;

    resp.assert_status_not_found();
    assert!(
        logs.contains("HTTP 404 Not Found response for GET /unique-middleware-not-found-route")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_set_target_creates_event_and_production_target(db_pool: SqlitePool) {
    let test_server = common::setup(db_pool.clone(), default_advisor());

    test_server
        .get("/set-target?reactor_name=pop&rate=1500.5")
        .await
        .assert_status_ok();

    let row = sqlx::query(
        "
            SELECT event.ingame_timestamp, production_target.rate
            FROM production_target
            JOIN event
            ON production_target.event_id = event.id
            JOIN reactor
            ON event.reactor_id = reactor.id
            WHERE reactor.name = ?
        ",
    )
    .bind("pop")
    .fetch_one(&db_pool)
    .await
    .unwrap();

    let ingame_timestamp: String = row.get("ingame_timestamp");
    let rate: f64 = row.get("rate");
    assert_valid_ingame_timestamp(&ingame_timestamp);
    assert_eq!(1500.5, rate);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_advice_uses_latest_stored_target_for_matching_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";

    let test_server = common::setup(db_pool.clone(), default_advisor());
    test_server
        .get("/set-target?reactor_name=pop&rate=1500.5")
        .await
        .assert_status_ok();
    test_server
        .get("/set-target?reactor_name=pop&rate=1600.25")
        .await
        .assert_status_ok();

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:00"))
        .advice(default_advice())
        .expected_response(default_expected_response(REACTOR_NAME))
        .check_target_burn_rate(|rate| assert_eq!(rate, 1600.25))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_advice_uses_request_target_without_stored_target(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const TARGET_ENERGY_PRODUCTION_RATE: f64 = 1234.5;

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:00"))
        .advice(default_advice())
        .expected_response(default_expected_response(REACTOR_NAME))
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_advice_targets_are_reactor_scoped(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const TARGET_ENERGY_PRODUCTION_RATE: f64 = 1234.5;

    let test_server = common::setup(db_pool.clone(), default_advisor());
    test_server
        .get("/set-target?reactor_name=other&rate=1600.25")
        .await
        .assert_status_ok();

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:00"))
        .advice(default_advice())
        .expected_response(default_expected_response(REACTOR_NAME))
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[must_use]
struct Test {
    reactor_name: Option<&'static str>,
    input: Option<Value>,
    check_past_events: Option<PastEventsCheck>,
    check_target_burn_rate: Option<TargetBurnRateCheck>,
    check_system_knowledge: Option<SystemKnowledgeCheck>,
    advice: Option<Advice>,
    expected_response: Option<Value>,
}

impl Test {
    fn new() -> Self {
        Self {
            reactor_name: None,
            input: None,
            check_past_events: None,
            check_target_burn_rate: None,
            check_system_knowledge: None,
            advice: None,
            expected_response: None,
        }
    }

    fn reactor_name(mut self, reactor_name: &'static str) -> Self {
        self.reactor_name = Some(reactor_name);
        self
    }

    fn input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }

    fn check_past_events(mut self, f: impl Fn(Vec<&PastEvent>) + Send + 'static) -> Self {
        self.check_past_events = Some(Box::new(f));
        self
    }

    fn check_target_burn_rate(mut self, f: impl Fn(f64) + Send + 'static) -> Self {
        self.check_target_burn_rate = Some(Box::new(f));
        self
    }

    fn check_system_knowledge(
        mut self,
        f: impl Fn(Option<&SystemKnowledge>) + Send + 'static,
    ) -> Self {
        self.check_system_knowledge = Some(Box::new(f));
        self
    }

    fn advice(mut self, advice: Advice) -> Self {
        self.advice = Some(advice);
        self
    }

    fn expected_response(mut self, expected_response: Value) -> Self {
        self.expected_response = Some(expected_response);
        self
    }

    async fn run(self, db_pool: SqlitePool) {
        let Self {
            reactor_name,
            input,
            check_past_events,
            check_target_burn_rate,
            check_system_knowledge,
            advice,
            expected_response,
        } = self;
        let reactor_name = reactor_name.expect("no reactor name");
        let input = input.expect("no json input");
        let expected_response = expected_response.expect("no expected response");

        let advisor = {
            MockAdvisor::new(move |reactor_states, target_burn_rate, knowledge| {
                let advice = advice.clone().expect("no advice");
                if let Some(check_past_events) = &check_past_events {
                    check_past_events(reactor_states);
                }

                if let Some(check_target_burn_rate) = &check_target_burn_rate {
                    check_target_burn_rate(target_burn_rate);
                }
                if let Some(check_system_knowledge) = &check_system_knowledge {
                    check_system_knowledge(knowledge);
                }
                Ok(advice)
            })
        };
        let test_server = common::setup(db_pool, advisor);

        let resp = test_server.post("/advice").json(&input).await;
        let body: Value = resp.json();
        assert_eq!(body["reactor_name"], reactor_name);
        assert_eq!(body, expected_response);
    }
}

fn active_request(reactor_name: &str, timestamp: &str) -> Value {
    json!({
        "reactor_name": reactor_name,
        "target_energy_production_rate": 1234.5,
        "reactor_state": {
            "integrity": "intact",
            "mode": "active",
            "temperature": 0.0,
            "coolant_filled_percent": 0.0,
            "heated_coolant_filled_percent": 0.0,
            "fuel_filled_percent": 0.0,
            "waste_filled_percent": 0.0,
            "actual_burn_rate": 0.0,
            "target_burn_rate": 0,
            "max_burn_rate": 1000,
            "damage_percent": 0.0,
            "heating_rate": 0.0,
            "boil_efficiency_percent": 0.0,
        },
        "turbine_state": {
            "integrity": "intact",
            "stored_kinetic_energy": 789.0,
            "energy_production_rate": 456.0,
        },
        "timestamp": timestamp,
    })
}

fn default_advisor() -> MockAdvisor {
    MockAdvisor::new(|_, _, _| Ok(default_advice()))
}

fn default_advice() -> Advice {
    Advice {
        action: AdvisedAction::NoAction,
        reasoning: "all good".into(),
        system_knowledge: SystemKnowledge::from("Stable baseline.".to_owned()),
    }
}

fn default_expected_response(reactor_name: &str) -> Value {
    json!({
        "reactor_name": reactor_name,
        "advice": {
            "action": {
                "kind": "no-action",
            },
            "reasoning": "all good",
            "system_knowledge": "Stable baseline.",
        },
    })
}

fn assert_valid_ingame_timestamp(timestamp: &str) {
    assert_eq!(timestamp.len(), 19);
    assert_eq!(&timestamp[4..5], "-");
    assert_eq!(&timestamp[7..8], "-");
    assert_eq!(&timestamp[10..11], "T");
    assert_eq!(&timestamp[13..14], ":");
    assert_eq!(&timestamp[16..17], ":");

    let year: u16 = timestamp[0..4].parse().unwrap();
    let month: u8 = timestamp[5..7].parse().unwrap();
    let day: u8 = timestamp[8..10].parse().unwrap();
    let hour: u8 = timestamp[11..13].parse().unwrap();
    let minute: u8 = timestamp[14..16].parse().unwrap();
    let second: u8 = timestamp[17..19].parse().unwrap();

    assert!(year > 2000);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));
    assert!(hour <= 23);
    assert!(minute <= 59);
    assert!(second <= 59);
}
