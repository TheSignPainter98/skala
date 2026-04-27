mod llm_advisor;

pub use self::llm_advisor::LlmAdvisor;

use std::fmt::Debug;

use crate::Result;
use crate::reactor::{ReactorState, TargetBurnRate};
use crate::time::IngameDateTime;

pub trait Advisor: Debug + Send + Sync {
    fn advise(
        &self,
        past_events: impl IntoIterator<Item = PastEvent> + Send,
    ) -> impl Future<Output = Result<Advice>> + Send;
}

#[derive(Clone, Debug)]
pub enum PastEvent {
    ReactorSnapshot(ReactorSnapshot),
    Action(PastAction),
}

impl PastEvent {
    pub(crate) fn timestamp(&self) -> &IngameDateTime {
        match self {
            Self::ReactorSnapshot(snapshot) => &snapshot.timestamp,
            Self::Action(action) => &action.timestamp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReactorSnapshot {
    pub timestamp: IngameDateTime,
    pub state: ReactorState,
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
    #[serde(skip)]
    Scram,

    /// Represents that the burn rate needs to be changed to the given value.
    #[serde(rename_all = "kebab-case")]
    SetBurnRate {
        /// The value of the new target burn rate.
        new_target_burn_rate: TargetBurnRate,
    },
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
