use std::fmt::Debug;

use anyhow::Context;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestDeveloperMessageArgs, ChatCompletionRequestDeveloperMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs, ReasoningEffort,
    ResponseFormat, ResponseFormatJsonSchema, Verbosity,
};
use serde_json::Value;

use crate::reactor::ReactorStatus;
use crate::{
    ConfigFrequencyPenalty, ConfigMaxCompletionTokens, ConfigPresencePenalty, ConfigTemperature,
    ConfigUrl, LlmConfig, Result, reactor::ReactorState,
};

pub(crate) trait Advisor: Debug + Send + Sync {
    fn advise(&self, reactor_state: ReactorState) -> impl Future<Output = Result<String>> + Send;
}

// TODO(kcza): move into advice module
#[derive(Clone, Debug)]
pub(crate) struct LlmAdvisor {
    client: Client<OpenAIConfig>,
    schemas: Schemas,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    max_completion_tokens: u32,
}

impl LlmAdvisor {
    pub(crate) fn new(config: LlmConfig) -> Self {
        let LlmConfig {
            url: ConfigUrl(url),
            temperature: ConfigTemperature(temperature),
            frequency_penalty: ConfigFrequencyPenalty(frequency_penalty),
            presence_penalty: ConfigPresencePenalty(presence_penalty),
            max_completion_tokens: ConfigMaxCompletionTokens(max_completion_tokens),
        } = config;

        let client_config = OpenAIConfig::new().with_api_base(url);
        let client = Client::with_config(client_config);

        let schemas = Schemas::new();
        Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
        }
    }
}

impl Advisor for LlmAdvisor {
    async fn advise(&self, reactor_state: ReactorState) -> Result<String> {
        // TODO(kcza): store previous messages in sqlite, present a sliding window of at
        // most N.

        let Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
        } = self;

        let messages = vec![
            ChatCompletionRequestMessage::Developer(
                ChatCompletionRequestDeveloperMessageArgs::default()
                    .name("god")
                    .content(ChatCompletionRequestDeveloperMessageContent::Text(
                        include_str!("base_prompt.md").to_owned(),
                    ))
                    .build()?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .name("reactor-monitor")
                    .content(ChatCompletionRequestUserMessageContent::Text(
                        self.summarise_reactor_state(&reactor_state),
                    ))
                    .build()?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .name("boss")
                    .content(ChatCompletionRequestUserMessageContent::Text(
                        "what do you recommend".to_owned(),
                    ))
                    .build()?,
            ),
        ];
        let response_format = ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                name: "reactor-control-commands".to_owned(),
                description: Some("reasoned reactor control commands".to_owned()),
                schema: Some(schemas.advice_response().clone()),
                strict: Some(true),
            },
        };
        let req = CreateChatCompletionRequestArgs::default()
            .messages(messages)
            .model("qwen-vl")
            .verbosity(Verbosity::Low)
            .reasoning_effort(ReasoningEffort::High)
            .max_completion_tokens(*max_completion_tokens)
            .frequency_penalty(*frequency_penalty)
            .presence_penalty(*presence_penalty)
            .response_format(response_format)
            .store(false)
            .stream(false)
            .n(1)
            .temperature(*temperature)
            .safety_identifier("skala")
            .build()?;
        let response = client.chat().create(req).await?;
        let raw_advice = response
            .choices
            .into_iter()
            .next()
            .context("llm returned too few choices")?
            .message
            .content
            .context("llm choice returned no content")?;
        Ok(raw_advice)
    }
}

impl LlmAdvisor {
    fn summarise_reactor_state(&self, reactor_state: &ReactorState) -> String {
        let ReactorState {
            status,
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
        } = reactor_state;
        let status = match status {
            ReactorStatus::Active => "active",
            ReactorStatus::Inactive => "inactive",
        };
        format!(
            "
                The reactor's status is {status}.
                The reactor's temperature {temperature}.
                The reactor's coolant_filled {coolant_filled}.
                The reactor's heated_coolant_filled {heated_coolant_filled}.
                The reactor's fuel_filled {fuel_filled}.
                The reactor's waste_filled {waste_filled}.
                The reactor's actual_burn_rate {actual_burn_rate}.
                The reactor's target_burn_rate {target_burn_rate}.
                The reactor's damage_percent {damage_percent}.
                The reactor's heating_rate {heating_rate}.
                The reactor's boil_efficiency {boil_efficiency}.
            "
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Schemas {
    advice_response: Value,
}

impl Schemas {
    pub(crate) fn new() -> Self {
        static RAW_ADVISE_RESPONSE_SCHEMA: &str = include_str!("llm-response-schema.json");
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
