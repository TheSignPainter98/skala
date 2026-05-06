mod common;

use serde_json::{Value, json};
use skala_server::advisor::{Advice, AdvisedAction, Insights, PastEvent, Snapshot};
use skala_server::{
    ActualBurnRate, IntactReactorSnapshot, IntactTurbineSnapshot, ReactorMode, ReactorSnapshot,
    TargetBurnRate, TurbineSnapshot,
};
use sqlx::SqlitePool;

use crate::common::MockAdvisor;

type PastEventsCheck = Box<dyn Fn(Vec<&PastEvent>) + Send>;
type TargetBurnRateCheck = Box<dyn Fn(f64) + Send>;
type InsightsCheck = Box<dyn Fn(Option<&Insights>) + Send>;

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
            insight_update: Some(Insights::from(
                "Inactive reactor remained stable.".to_owned(),
            )),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "all good",
                "insight_update": "Inactive reactor remained stable.",
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
        .check_insights(|insights| assert!(insights.is_none()))
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
            insight_update: Some(Insights::from(
                "Initial active response requested more burn.".to_owned(),
            )),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "set-burn-rate",
                    "new_target_burn_rate": 1000,
                },
                "reasoning": "let's see what happens",
                "insight_update": "Initial active response requested more burn.",
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
        .check_insights(|insights| assert!(insights.is_none()))
        .check_target_burn_rate(|rate| assert_eq!(rate, TARGET_ENERGY_PRODUCTION_RATE))
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_previous_insights_are_passed_to_advisor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";
    const FIRST_INSIGHTS: &str = "A 100 mL/s burn-rate increase lifted output after one snapshot.";
    const UPDATED_INSIGHTS: &str = "Holding the new burn rate kept output stable.";

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:00"))
        .advice(Advice {
            action: AdvisedAction::NoAction,
            reasoning: "collect baseline".into(),
            insight_update: Some(Insights::from(FIRST_INSIGHTS.to_owned())),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "collect baseline",
                "insight_update": FIRST_INSIGHTS,
            },
        }))
        .check_insights(|insights| assert!(insights.is_none()))
        .run(db_pool.clone())
        .await;

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(active_request(REACTOR_NAME, "2026-04-15T00:00:01"))
        .advice(Advice {
            action: AdvisedAction::NoAction,
            reasoning: "hold steady".into(),
            insight_update: Some(Insights::from(UPDATED_INSIGHTS.to_owned())),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": {
                    "kind": "no-action",
                },
                "reasoning": "hold steady",
                "insight_update": UPDATED_INSIGHTS,
            },
        }))
        .check_insights(|insights| {
            let insights = insights.cloned().map(String::from);
            assert_eq!(insights.as_deref(), Some(FIRST_INSIGHTS));
        })
        .run(db_pool)
        .await;
}

#[must_use]
struct Test {
    reactor_name: Option<&'static str>,
    input: Option<Value>,
    check_past_events: Option<PastEventsCheck>,
    check_target_burn_rate: Option<TargetBurnRateCheck>,
    check_insights: Option<InsightsCheck>,
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
            check_insights: None,
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

    fn check_insights(mut self, f: impl Fn(Option<&Insights>) + Send + 'static) -> Self {
        self.check_insights = Some(Box::new(f));
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
            check_insights,
            advice,
            expected_response,
        } = self;
        let reactor_name = reactor_name.expect("no reactor name");
        let input = input.expect("no json input");
        let expected_response = expected_response.expect("no expected response");

        let advisor = {
            MockAdvisor::new(move |reactor_states, target_burn_rate, insights| {
                let advice = advice.clone().expect("no advice");
                if let Some(check_past_events) = &check_past_events {
                    check_past_events(reactor_states);
                }

                if let Some(check_target_burn_rate) = &check_target_burn_rate {
                    check_target_burn_rate(target_burn_rate);
                }
                if let Some(check_insights) = &check_insights {
                    check_insights(insights);
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
        "timestamp": timestamp,
    })
}
