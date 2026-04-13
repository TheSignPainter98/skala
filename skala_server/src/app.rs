use std::ops::Deref;
use std::sync::Arc;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use axum::Router;
use serde_json::Value;
use sqlx::SqlitePool;

use crate::{
    Config, ConfigFrequencyPenalty, ConfigMaxCompletionTokens, ConfigPresencePenalty,
    ConfigTemperature, ConfigUrl, LlmConfig, routes,
};

pub(crate) struct App {
    router: Router<()>,
}

impl App {
    pub(crate) fn new(config: Config, db_pool: SqlitePool, advisor: ()) -> Self {
        let app_state = AppState::new(config, db_pool, advisor);
        let router = routes::register(Router::new()).with_state(app_state);
        Self { router }
    }

    pub(crate) fn into_router(self) -> Router<()> {
        let Self { router } = self;
        router
    }
}
#[derive(Clone, Debug)]
pub(crate) struct AppState(Arc<AppStateInner>);

impl AppState {
    fn new(config: Config, db_pool: SqlitePool, _advisor: ()) -> Self {
        let Config {
            llm: llm_config, ..
        } = config;
        let LlmConfig {
            url: ConfigUrl(url),
            temperature: ConfigTemperature(temperature),
            frequency_penalty: ConfigFrequencyPenalty(frequency_penalty),
            presence_penalty: ConfigPresencePenalty(presence_penalty),
            max_completion_tokens: ConfigMaxCompletionTokens(max_completion_tokens),
        } = llm_config;

        let llm_config = OpenAIConfig::new().with_api_base(url);
        let llm_client = Client::with_config(llm_config);

        let schemas = Schemas::new();
        Self(Arc::new(AppStateInner {
            db_pool,
            advisor: (),
            llm_client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
        }))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) db_pool: SqlitePool,
    #[allow(unused)]
    pub(crate) advisor: (),
    pub(crate) llm_client: Client<OpenAIConfig>,
    pub(crate) schemas: Schemas,
    pub(crate) temperature: f32,
    pub(crate) frequency_penalty: f32,
    pub(crate) presence_penalty: f32,
    pub(crate) max_completion_tokens: u32,
}

#[derive(Debug)]
pub(crate) struct Schemas {
    advice_response: Value,
}

impl Schemas {
    fn new() -> Self {
        static RAW_ADVISE_RESPONSE_SCHEMA: &str =
            include_str!("routes/advice/llm-response-schema.json");
        let advice_response =
            serde_json::from_str(RAW_ADVISE_RESPONSE_SCHEMA).expect("cannot parse schema");
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
