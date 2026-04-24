mod common;

use serde_json::{Value, json};
use skala_server::advisor::{Advice, AdvisedAction, ReactorSnapshot};
use skala_server::{IntactReactorState, ReactorMode, ReactorState};
use sqlx::SqlitePool;

use crate::common::MockAdvisor;

#[sqlx::test(migrations = "./migrations")]
async fn test_destroyed_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "reactor_state": {
                "integrity": "destroyed"
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

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "reactor_state": {
                "integrity": "intact",
                "mode": "inactive",
                "temperature": 111.0,
                "coolant_filled": 222.0,
                "heated_coolant_filled": 333.0,
                "fuel_filled": 444.0,
                "waste_filled": 555.0,
                "actual_burn_rate": 666.0,
                "target_burn_rate": 777.0,
                "damage_percent": 888.0,
                "heating_rate": 999.0,
                "boil_efficiency": 1234.0,
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
                "action": "no-action",
                "reasoning": "all good",
            },
        }))
        .check_reactor_snapshots(|snapshots| {
            let snapshot = match snapshots.as_slice() {
                [] => panic!("too few snapshots"),
                [snapshot] => snapshot.to_owned(),
                _ => panic!("too many snapshots"),
            };
            let ReactorSnapshot { timestamp, state } = snapshot;
            assert_eq!(timestamp.into_inner(), TIMESTAMP);
            let intact_reactor_state = match state {
                ReactorState::Intact(intact_reactor_state) => intact_reactor_state,
                _ => panic!("unexpected reactor state"),
            };
            let IntactReactorState {
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
            } = intact_reactor_state;
            assert!(matches!(mode, ReactorMode::Inactive));
            assert_eq!(temperature, 111.0);
            assert_eq!(coolant_filled, 222.0);
            assert_eq!(heated_coolant_filled, 333.0);
            assert_eq!(fuel_filled, 444.0);
            assert_eq!(waste_filled, 555.0);
            assert_eq!(actual_burn_rate, 666.0);
            assert_eq!(target_burn_rate, 777.0);
            assert_eq!(damage_percent, 888.0);
            assert_eq!(heating_rate, 999.0);
            assert_eq!(boil_efficiency, 1234.0);
        })
        .run(db_pool)
        .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_active_reactor(db_pool: SqlitePool) {
    const REACTOR_NAME: &str = "pop";

    Test::new()
        .reactor_name(REACTOR_NAME)
        .input(json!({
            "reactor_name": REACTOR_NAME,
            "reactor_state": {
                "integrity": "intact",
                "mode": "active",
                "temperature": 0.0,
                "coolant_filled": 0.0,
                "heated_coolant_filled": 0.0,
                "fuel_filled": 0.0,
                "waste_filled": 0.0,
                "actual_burn_rate": 0.0,
                "target_burn_rate": 0.0,
                "damage_percent": 0.0,
                "heating_rate": 0.0,
                "boil_efficiency": 0.0,
            },
            "timestamp": "2026-04-15T00:00:00"
        }))
        .advice(Advice {
            action: AdvisedAction::Scram,
            reasoning: "let's see what happens".into(),
        })
        .expected_response(json!({
            "reactor_name": REACTOR_NAME,
            "advice": {
                "action": "scram",
                "reasoning": "let's see what happens",
            },
        }))
        .check_reactor_snapshots(|snapshots| {
            assert!(matches!(
                snapshots.as_slice(),
                [ReactorSnapshot {
                    state: ReactorState::Intact(IntactReactorState {
                        mode: ReactorMode::Active,
                        ..
                    }),
                    ..
                }],
            ));
        })
        .run(db_pool)
        .await;
}

#[must_use]
struct Test {
    reactor_name: Option<&'static str>,
    input: Option<Value>,
    check_reactor_snapshots: Option<Box<dyn Fn(Vec<ReactorSnapshot>) + Send>>,
    advice: Option<Advice>,
    expected_response: Option<Value>,
}

impl Test {
    fn new() -> Self {
        Self {
            reactor_name: None,
            input: None,
            check_reactor_snapshots: None,
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

    fn check_reactor_snapshots(
        mut self,
        f: impl Fn(Vec<ReactorSnapshot>) + Send + 'static,
    ) -> Self {
        self.check_reactor_snapshots = Some(Box::new(f));
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
            check_reactor_snapshots: check_reactor_states,
            advice,
            expected_response,
        } = self;
        let reactor_name = reactor_name.expect("no reactor name");
        let input = input.expect("no json input");
        let expected_response = expected_response.expect("no expected response");

        let advisor = {
            MockAdvisor::new(move |reactor_states| {
                let advice = advice.clone().expect("no advice");
                let check_reactor_states = check_reactor_states
                    .as_ref()
                    .expect("no reactor state check");
                check_reactor_states(reactor_states);
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
