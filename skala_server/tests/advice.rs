mod common;

use serde_json::{Value, json};
use skala_server::advisor::{Advice, AdvisedAction, PastEvent, Snapshot};
use skala_server::{
    ActualBurnRate, IntactReactorSnapshot, IntactTurbineSnapshot, ReactorMode, ReactorSnapshot,
    TargetBurnRate, TurbineSnapshot,
};
use sqlx::SqlitePool;

use crate::common::MockAdvisor;

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
                "coolant_filled": 222.0,
                "heated_coolant_filled": 333.0,
                "fuel_filled": 444.0,
                "waste_filled": 555.0,
                "actual_burn_rate": 666.0,
                "target_burn_rate": 777,
                "damage_percent": 888.0,
                "heating_rate": 999.0,
                "boil_efficiency": 1234.0,
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
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "all good",
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
                coolant_filled,
                heated_coolant_filled,
                fuel_filled,
                waste_filled,
                actual_burn_rate,
                target_burn_rate,
                damage_percent,
                heating_rate,
                boil_efficiency,
            } = intact_reactor_snapshot;
            assert!(matches!(*mode, ReactorMode::Inactive));
            assert_eq!(*temperature, 111.0);
            assert_eq!(*coolant_filled, 222.0);
            assert_eq!(*heated_coolant_filled, 333.0);
            assert_eq!(*fuel_filled, 444.0);
            assert_eq!(*waste_filled, 555.0);
            assert_eq!(*actual_burn_rate, ActualBurnRate::from(666.0));
            assert_eq!(*target_burn_rate, TargetBurnRate::from(777));
            assert_eq!(*damage_percent, 888.0);
            assert_eq!(*heating_rate, 999.0);
            assert_eq!(*boil_efficiency, 1234.0);

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
                "coolant_filled": 0.0,
                "heated_coolant_filled": 0.0,
                "fuel_filled": 0.0,
                "waste_filled": 0.0,
                "actual_burn_rate": 0.0,
                "target_burn_rate": 0,
                "damage_percent": 0.0,
                "heating_rate": 0.0,
                "boil_efficiency": 0.0,
            },
            "turbine_state": {
                "integrity": "intact",
                "stored_kinetic_energy": 789.0,
                "energy_production_rate": 456.0,
            },
            "timestamp": "2026-04-15T00:00:00"
        }))
        .advice(Advice {
            action: AdvisedAction::SetBurnRate {
                new_target_burn_rate: 1000.into(),
            },
            reasoning: "let's see what happens".into(),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "set-burn-rate",
                    "new_target_burn_rate": 1000,
                },
                "reasoning": "let's see what happens",
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
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[must_use]
struct Test {
    reactor_name: Option<&'static str>,
    input: Option<Value>,
    #[allow(clippy::type_complexity)]
    check_past_events: Option<Box<dyn Fn(Vec<&PastEvent>) + Send>>,
    check_target_burn_rate: Option<Box<dyn Fn(f64) + Send>>,
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
            advice,
            expected_response,
        } = self;
        let reactor_name = reactor_name.expect("no reactor name");
        let input = input.expect("no json input");
        let expected_response = expected_response.expect("no expected response");

        let advisor = {
            MockAdvisor::new(move |reactor_states, target_burn_rate| {
                let advice = advice.clone().expect("no advice");
                let check_past_events = check_past_events.as_ref().expect("no reactor state check");
                check_past_events(reactor_states);

                if let Some(check_target_burn_rate) = &check_target_burn_rate {
                    check_target_burn_rate(target_burn_rate);
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
