pub mod backend;
mod biased_alternate;
pub mod feedback;

use std::iter;

use indoc::formatdoc;
use log::info;
use schemars::schema_for;
use serde_json::Value;

use crate::advisor::feedback::FeedbackProvider;
use crate::advisor::llm_advisor::backend::Backend;
use crate::advisor::llm_advisor::biased_alternate::BiasedAlternateWithExt;
use crate::advisor::{Advice, Advisor, PastAction, PastEvent, Snapshot, SystemKnowledge};
use crate::{Feedback, LlmConfig, ReactorSnapshot, Result, TurbineSnapshot};

#[derive(Clone, Debug)]
pub struct LlmAdvisor {
    backend: Backend,
    feedback_provider: FeedbackProvider,
}

impl LlmAdvisor {
    pub fn new(config: LlmConfig, backend: impl Into<Backend>) -> Self {
        let LlmConfig {
            url: _,
            temperature: _,
            frequency_penalty: _,
            presence_penalty: _,
            max_completion_tokens: _,
            feedback_regime,
            feedback,
        } = config;

        let backend = backend.into();
        let feedback_provider = FeedbackProvider::new(feedback_regime, feedback);
        Self {
            backend,
            feedback_provider,
        }
    }

    // TODO(kcza): make `TargetEnergyProductionRate` type
    fn prompt_info<'info>(
        &'info self,
        past_events: impl IntoIterator<Item = &'info PastEvent>,
        target_energy_production_rate: f64,
        system_knowledge: Option<&'info SystemKnowledge>,
    ) -> impl Iterator<Item = PromptInfo<'info>> {
        let past_events = past_events.into_iter().map(|event| match event {
            PastEvent::Snapshot(snapshot) => PromptInfo::Snapshot(snapshot),
            PastEvent::PastAction(action) => PromptInfo::PastAction(action),
        });
        let feedback = self.feedback_provider.feedback().map(PromptInfo::Feedback);
        let system_knowledge = iter::once(
            system_knowledge
                .map(PromptInfo::SystemKnowledge)
                .unwrap_or(PromptInfo::MissingSystemKnowledge),
        );
        let target = iter::once(PromptInfo::TargetEnergyProductionRate(
            target_energy_production_rate,
        ));
        iter::once(PromptInfo::Raw(include_str!("base_prompt.md")))
            .chain(
                past_events
                    .biased_alternate_with(feedback)
                    .on(|event| matches!(event, PromptInfo::PastAction(_))),
            )
            .chain(iter::once(PromptInfo::Raw(include_str!(
                "reactor_info_prompt.md"
            ))))
            .chain(system_knowledge)
            .chain(target)
    }
}

impl Advisor for LlmAdvisor {
    async fn advise<'event, I>(
        &'event self,
        past_events: I,
        target_energy_production_rate: f64,
        system_knowledge: Option<&'event SystemKnowledge>,
    ) -> Result<Advice>
    where
        I: IntoIterator<Item = &'event PastEvent> + Send,
        I::IntoIter: Send,
    {
        info!("creating message");
        let prompt_info =
            self.prompt_info(past_events, target_energy_production_rate, system_knowledge);

        info!("awaiting llm response");
        let ret = self.backend.fetch(prompt_info).await?;
        info!("returning llm response");
        Ok(ret)
    }
}

pub(crate) enum PromptInfo<'msg> {
    Raw(&'msg str),
    Snapshot(&'msg Snapshot),
    PastAction(&'msg PastAction),
    Feedback(&'msg Feedback),
    SystemKnowledge(&'msg SystemKnowledge),
    MissingSystemKnowledge,
    TargetEnergyProductionRate(f64),
}

impl PromptInfo<'_> {
    pub(crate) fn summary(&self) -> String {
        match self {
            Self::Raw(text) => (*text).to_owned(), // Forces unnecessary allocation.
            Self::Snapshot(snapshot) => {
                let Snapshot {
                    timestamp,
                    reactor,
                    turbine,
                } = snapshot;
                if matches!(reactor, ReactorSnapshot::Destroyed) {
                    return formatdoc!(
                        "
                            # Reactor state at {timestamp}

                            Reactor sensors malfunctioned; no data available.
                        "
                    );
                }

                let snapshot_body = SnapshotBody { reactor, turbine };
                let state_json = serde_json::to_string(&snapshot_body).unwrap();
                return formatdoc!(
                    "
                        # System state at {timestamp}

                        ```json
                        {state_json}
                        ```
                    "
                );

                // Serde types
                #[derive(serde::Serialize)]
                struct SnapshotBody<'body> {
                    reactor: &'body ReactorSnapshot,
                    turbine: &'body TurbineSnapshot,
                }
            }
            Self::PastAction(action) => {
                let PastAction {
                    timestamp,
                    action,
                } = action;
                let state_json = serde_json::to_string(action).unwrap();
                formatdoc!(
                    "
                        # Action taken at {timestamp}

                        ```json
                        {state_json}
                        ```
                    "
                )
            }
            Self::Feedback(feedback) => formatdoc! {
                "
                    # Feedback

                    {feedback}
                "
            },
            Self::SystemKnowledge(system_knowledge) => {
                let quoted_system_knowledge = system_knowledge.as_str()
                    .lines()
                    .collect::<Vec<_>>()
                    .join("\n> ");
                formatdoc! {
                    "
                        # Insights

                        > {quoted_system_knowledge}
                    "
                }
            },
            Self::MissingSystemKnowledge => {
                formatdoc! {"
                    # System knowledge

                    No inferred knowledge recorded yet. Set the `system_knowledge` field in the output.
                "}
            }
            Self::TargetEnergyProductionRate(target) => formatdoc!(
                "
                    # Your current goal

                    The national power grid controller kindly asks that we get the turbine's energy production rate to {target}, **safely**. What do you recommend?
                "
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Schemas {
    advice_response: Value,
}

impl Schemas {
    pub(crate) fn new() -> Self {
        let advice_response = schema_for!(Advice).to_value();
        Self { advice_response }
    }

    pub(crate) fn advice_response(&self) -> &Value {
        &self.advice_response
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::advisor::llm_advisor::backend::OpenAiBackend;
    use crate::advisor::{AdvisedAction, PastAction, Snapshot, SystemKnowledge};
    use crate::components::reactor::{
        ActualBurnRate, IntactReactorSnapshot, MaxBurnRate, Percent, ReactorMode, ReactorSnapshot,
        TargetBurnRate,
    };
    use crate::components::turbine::{IntactTurbineSnapshot, TurbineSnapshot};
    use crate::time::IngameDateTime;
    use crate::{FeedbackConfig, FeedbackRegime, LlmConfig};

    use super::*;

    #[test]
    fn test_schemas_valid() {
        // The following call panics on an invalid schema.
        Schemas::new();
    }

    #[test]
    fn test_prompt_info_summary_base_prompt() {
        let summary = PromptInfo::Raw("# Keep the reactor stable.").summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_llm_advisor_prompt_info_sequence() {
        let config = LlmConfig {
            feedback_regime: FeedbackRegime::Positive,
            feedback: FeedbackConfig {
                positive: vec![Feedback::new("Good job lad".to_owned())],
                negative: vec![Feedback::new("Och, no!".to_owned())],
            },
            ..Default::default()
        };
        let backend = OpenAiBackend::new(&config);
        let advisor = LlmAdvisor::new(config, backend);
        let system_knowledge = SystemKnowledge::from(
            "Burn-rate increases took two snapshots to affect turbine output.\nAvoid rapid oscillation."
                .to_owned(),
        );
        let past_events = vec![
            PastEvent::Snapshot(Snapshot {
                timestamp: IngameDateTime::from("2026-05-03T16:24:11".to_owned()),
                reactor: ReactorSnapshot::Intact(IntactReactorSnapshot {
                    mode: ReactorMode::Active,
                    temperature: 742.5,
                    coolant_filled_percent: Percent::from(0.82),
                    heated_coolant_filled_percent: Percent::from(0.34),
                    fuel_filled_percent: Percent::from(0.76),
                    waste_filled_percent: Percent::from(0.12),
                    actual_burn_rate: ActualBurnRate::from(503.2),
                    target_burn_rate: TargetBurnRate::from(500),
                    max_burn_rate: MaxBurnRate::from(1_000),
                    damage_percent: Percent::from(0.0),
                    heating_rate: 18.25,
                    boil_efficiency_percent: Percent::from(0.91),
                }),
                turbine: TurbineSnapshot::Intact(IntactTurbineSnapshot {
                    stored_kinetic_energy: 12_345.0,
                    energy_production_rate: 987.6,
                }),
            }),
            PastEvent::PastAction(PastAction {
                timestamp: IngameDateTime::from("2026-05-03T16:24:12".to_owned()),
                action: AdvisedAction::SetTargetBurnRate {
                    new_target_burn_rate: TargetBurnRate::from(650),
                },
            }),
            PastEvent::Snapshot(Snapshot {
                timestamp: IngameDateTime::from("2026-05-03T16:24:13".to_owned()),
                reactor: ReactorSnapshot::Destroyed,
                turbine: TurbineSnapshot::Destroyed,
            }),
            PastEvent::PastAction(PastAction {
                timestamp: IngameDateTime::from("2026-05-03T16:24:14".to_owned()),
                action: AdvisedAction::SetTargetBurnRate {
                    new_target_burn_rate: TargetBurnRate::from(300),
                },
            }),
        ];
        let prompt_repr = advisor
            .prompt_info(&past_events, 1_250.0, Some(&system_knowledge))
            .map(|info| info.summary())
            .collect::<Vec<_>>()
            .join("\n---\n");
        assert_snapshot!(prompt_repr);
    }

    #[test]
    fn test_prompt_info_summary_intact_snapshot() {
        let summary = PromptInfo::Snapshot(&Snapshot {
            timestamp: IngameDateTime::from("2026-05-03T16:24:11".to_owned()),
            reactor: ReactorSnapshot::Intact(IntactReactorSnapshot {
                mode: ReactorMode::Active,
                temperature: 742.5,
                coolant_filled_percent: Percent::from(0.82),
                heated_coolant_filled_percent: Percent::from(0.34),
                fuel_filled_percent: Percent::from(0.76),
                waste_filled_percent: Percent::from(0.12),
                actual_burn_rate: ActualBurnRate::from(503.2),
                target_burn_rate: TargetBurnRate::from(500),
                max_burn_rate: MaxBurnRate::from(1_000),
                damage_percent: Percent::from(0.0),
                heating_rate: 18.25,
                boil_efficiency_percent: Percent::from(0.91),
            }),
            turbine: TurbineSnapshot::Intact(IntactTurbineSnapshot {
                stored_kinetic_energy: 12_345.0,
                energy_production_rate: 987.6,
            }),
        })
        .summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_destroyed_snapshot() {
        let snapshot = Snapshot {
            timestamp: IngameDateTime::from("2026-05-03T16:24:11".to_owned()),
            reactor: ReactorSnapshot::Destroyed,
            turbine: TurbineSnapshot::Destroyed,
        };
        let summary = PromptInfo::Snapshot(&snapshot).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_past_action() {
        let action = PastAction {
            timestamp: IngameDateTime::from("2026-05-03T16:24:11".to_owned()),
            action: AdvisedAction::SetTargetBurnRate {
                new_target_burn_rate: TargetBurnRate::from(750),
            },
        };
        let summary = PromptInfo::PastAction(&action).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_feedback() {
        let feedback = Feedback::new("yer doin' good, lad".to_owned());
        let summary = PromptInfo::Feedback(&feedback).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_system_knowledge() {
        let system_knowledge = SystemKnowledge::from(
            "Burn-rate increases took two snapshots to affect turbine output.\nAvoid rapid oscillation."
                .to_owned(),
        );
        let summary = PromptInfo::SystemKnowledge(&system_knowledge).summary();

        assert!(summary.starts_with('#'));
        assert!(
            summary.contains("> Burn-rate increases took two snapshots to affect turbine output.")
        );
        assert!(summary.contains("> Avoid rapid oscillation."));
    }

    #[test]
    fn test_prompt_info_summary_target_energy_production_rate() {
        let summary = PromptInfo::TargetEnergyProductionRate(1_000.0).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }
}
