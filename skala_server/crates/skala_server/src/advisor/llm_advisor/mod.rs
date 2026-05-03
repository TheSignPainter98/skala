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
                    timestamp: _,
                    action,
                } = action;
                serde_json::to_string(action).unwrap()
            }
            Self::Feedback(feedback) => feedback.to_string(),
            Self::TargetEnergyProductionRate(target) => format!(
                "We have to get the turbine's energy production rate to {target}. What do you recommend?"
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
    use super::*;

    #[test]
    fn test_schemas_valid() {
        // The following call panics on an invalid schema.
        Schemas::new();
    }
}
