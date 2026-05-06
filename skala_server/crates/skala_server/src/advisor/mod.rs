mod llm_advisor;

pub use self::llm_advisor::LlmAdvisor;
pub use self::llm_advisor::backend::{Backend, CopyPasteBackend, OpenAiBackend};
pub use self::llm_advisor::feedback;

use std::fmt::Debug;

use crate::Result;
use crate::components::reactor::{ReactorSnapshot, TargetBurnRate};
use crate::components::turbine::TurbineSnapshot;
use crate::time::IngameDateTime;

pub trait Advisor: Debug + Send + Sync {
    fn advise<'event, I>(
        &'event self,
        past_events: I,
        target_energy_production_rate: f64,
        insights: Option<&'event Insights>,
    ) -> impl Future<Output = Result<Advice>> + Send
    where
        I: IntoIterator<Item = &'event PastEvent> + Send,
        I::IntoIter: Send;
}

#[derive(Clone, Debug)]
pub enum PastEvent {
    Snapshot(Snapshot),
    PastAction(PastAction),
}

impl PastEvent {
    pub(crate) fn timestamp(&self) -> &IngameDateTime {
        match self {
            Self::Snapshot(snapshot) => &snapshot.timestamp,
            Self::PastAction(action) => &action.timestamp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub timestamp: IngameDateTime,
    pub reactor: ReactorSnapshot,
    pub turbine: TurbineSnapshot,
}

#[derive(Clone, Debug)]
pub struct PastAction {
    pub timestamp: IngameDateTime,
    pub action: AdvisedAction,
}

/// Holds the advice to apply to the reactor.
#[derive(
    Clone, Debug, quicktype::Quicktype, schemars::JsonSchema, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub struct Advice {
    /// The best course of action.
    pub action: AdvisedAction,

    /// A very concise description of the reasoning behind the best course of action. The reasoning description must contain at most 16 words.
    #[schemars(length(max = 120))]
    pub reasoning: String,

    /// The updated insights into how the system works. If nothing useful has been learned,
    /// set this to the quoted portion of the 'Past insights' section you were passed in your
    /// prompt.
    ///
    /// If more has been understood about the system set the new 'insights' here. If present, these
    /// insights will be passed to you the next time you need to operate the reactor. You will
    /// forget all information you do not place here. Setting this field is the only way you have to
    /// learn about the system over the long term. If this field is unset, it means that you believe
    /// that your current insights are sufficient to operate the reactor safely.
    pub insight_update: Option<Insights>,
}

/// Holds the action to apply to the reactor.
#[derive(
    Clone, Debug, quicktype::Quicktype, schemars::JsonSchema, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "kind")]
#[quicktype(namespace = "server")]
pub enum AdvisedAction {
    /// Represents that the reactor's state is okay and hence that no action is required.
    NoAction,

    /// Represents that the reactor is either critical or will soon go critical, hence the reaction must be stopped immediately.
    Scram,

    /// Represents that the burn rate needs to be changed to the given value.
    #[serde(rename = "set-burn-rate")]
    SetBurnRate {
        /// The value of the new target burn rate.
        new_target_burn_rate: TargetBurnRate,
    },
}

#[derive(
    Clone,
    Debug,
    quicktype::Quicktype,
    schemars::JsonSchema,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Insights(String);

impl Insights {
    pub(crate) fn as_str(&self) -> &str {
        let Self(inner) = self;
        inner
    }
}

impl From<String> for Insights {
    fn from(inner: String) -> Self {
        Self(inner)
    }
}

impl From<Insights> for String {
    fn from(insights: Insights) -> Self {
        let Insights(inner) = insights;
        inner
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use schemars::schema_for;

    use super::*;

    #[googletest::test]
    fn test_schema_consistency() {
        assert_json_snapshot!(schema_for!(Advice));
    }
}
