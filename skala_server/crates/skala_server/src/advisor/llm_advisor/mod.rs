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
use crate::advisor::{Advice, Advisor, PastAction, PastEvent, Snapshot};
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
}

impl Advisor for LlmAdvisor {
    async fn advise<'event, I>(
        &self,
        past_events: I,
        target_energy_production_rate: f64,
    ) -> Result<Advice>
    where
        I: IntoIterator<Item = &'event PastEvent> + Send,
        I::IntoIter: Send,
    {
        let Self {
            backend,
            feedback_provider,
        } = self;

        info!("creating message");
        let past_events = past_events.into_iter().map(|event| match event {
            PastEvent::Snapshot(snapshot) => PromptInfo::Snapshot(snapshot),
            PastEvent::PastAction(action) => PromptInfo::PastAction(action),
        });
        let feedback = feedback_provider.feedback().map(PromptInfo::Feedback);
        let target = iter::once(PromptInfo::TargetEnergyProductionRate(
            target_energy_production_rate,
        ));
        let prompt_info = iter::once(PromptInfo::BasePrompt(include_str!("base_prompt.md")))
            .chain(past_events.biased_alternate_with(feedback))
            .chain(target);

        info!("awaiting llm response");
        let ret = backend.fetch(prompt_info).await?;
        Ok(ret)
    }
}

pub(crate) enum PromptInfo<'msg> {
    BasePrompt(&'msg str),
    Snapshot(&'msg Snapshot),
    PastAction(&'msg PastAction),
    Feedback(&'msg Feedback),
    TargetEnergyProductionRate(f64),
}

impl PromptInfo<'_> {
    pub(crate) fn summary(&self) -> String {
        match self {
            Self::BasePrompt(text) => (*text).to_owned(), // Forces unnecessary allocation.
            Self::Snapshot(snapshot) => {
                let Snapshot {
                    timestamp,
                    reactor,
                    turbine,
                } = snapshot;
                if matches!(reactor, ReactorSnapshot::Destroyed) {
                    return formatdoc!(
                        "
                            ### Reactor state at {timestamp}

                            Reactor sensors malfunctioned; no data available.
                        "
                    );
                }

                let snapshot_body = SnapshotBody { reactor, turbine };
                let state_json = serde_json::to_string(&snapshot_body).unwrap();
                return formatdoc!(
                    "
                        ### System state at {timestamp}

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
                        ### Action taken at {timestamp}

                        ```json
                        {state_json}
                        ```
                    "
                )
            }
            Self::Feedback(feedback) => feedback.to_string(),
            Self::TargetEnergyProductionRate(target) => formatdoc!(
                "
                    # Your current goal

                    We have to get the turbine's energy production rate to {target}. What do you recommend?
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

    use crate::advisor::{AdvisedAction, PastAction, Snapshot};
    use crate::components::reactor::{
        ActualBurnRate, IntactReactorSnapshot, ReactorMode, ReactorSnapshot, TargetBurnRate,
    };
    use crate::components::turbine::{IntactTurbineSnapshot, TurbineSnapshot};
    use crate::time::IngameDateTime;

    use super::*;

    #[test]
    fn test_schemas_valid() {
        // The following call panics on an invalid schema.
        Schemas::new();
    }

    #[test]
    fn test_prompt_info_summary_base_prompt() {
        let summary = PromptInfo::BasePrompt("# Keep the reactor stable.").summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_intact_snapshot() {
        let summary = PromptInfo::Snapshot(&Snapshot {
            timestamp: IngameDateTime::from("2026-05-03T16:24:11".to_owned()),
            reactor: ReactorSnapshot::Intact(IntactReactorSnapshot {
                mode: ReactorMode::Active,
                temperature: 742.5,
                coolant_filled: 0.82,
                heated_coolant_filled: 0.34,
                fuel_filled: 0.76,
                waste_filled: 0.12,
                actual_burn_rate: ActualBurnRate::from(503.2),
                target_burn_rate: TargetBurnRate::from(500),
                damage_percent: 0.0,
                heating_rate: 18.25,
                boil_efficiency: 0.91,
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
            action: AdvisedAction::SetBurnRate {
                new_target_burn_rate: TargetBurnRate::from(750),
            },
        };
        let summary = PromptInfo::PastAction(&action).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_feedback() {
        let feedback: Feedback =
            serde_json::from_str(r##"{"content":"# Burn rate corrections stabilized output."}"##)
                .unwrap();
        let summary = PromptInfo::Feedback(&feedback).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }

    #[test]
    fn test_prompt_info_summary_target_energy_production_rate() {
        let summary = PromptInfo::TargetEnergyProductionRate(1_000.0).summary();

        assert!(summary.starts_with('#'));
        assert_snapshot!(summary);
    }
}
