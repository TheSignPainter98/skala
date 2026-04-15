mod llm_advisor;

pub use self::llm_advisor::LlmAdvisor;

use std::fmt::Debug;

use crate::Result;
use crate::reactor::IntactReactorState;

pub trait Advisor: Debug + Send + Sync {
    fn advise(
        &self,
        reactor_state: IntactReactorState,
    ) -> impl Future<Output = Result<Advice>> + Send;
}

/// Holds the advice to apply to the reactor.
#[derive(Clone, Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Advice {
    pub action: AdvisedAction,
    pub reasoning: String,
}

/// Holds the action to apply to the reactor.
#[derive(Clone, Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisedAction {
    /// Represents that the reactor's state is okay and hence that no action is required.
    NoAction,
    /// Represents that the reactor is either critical or will soon go critical, hence the reaction must be stopped immediately.
    Scram,
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
