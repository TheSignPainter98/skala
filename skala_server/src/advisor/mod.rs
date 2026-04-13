mod llm_advisor;

pub(crate) use self::llm_advisor::LlmAdvisor;

use std::fmt::Debug;

use crate::Result;
use crate::reactor::ReactorState;

pub(crate) trait Advisor: Debug + Send + Sync {
    fn advise(&self, reactor_state: ReactorState) -> impl Future<Output = Result<String>> + Send;
}
