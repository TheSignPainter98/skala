use anyhow::Context;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestDeveloperMessageArgs, ChatCompletionRequestDeveloperMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs, ReasoningEffort,
    ResponseFormat, ResponseFormatJsonSchema, Verbosity,
};
use indoc::formatdoc;
use schemars::schema_for;
use serde_json::Value;

use crate::advisor::{Advice, Advisor, ReactorSnapshot};
use crate::reactor::{IntactReactorState, ReactorMode, ReactorState};
use crate::{
    ConfigFrequencyPenalty, ConfigMaxCompletionTokens, ConfigPresencePenalty, ConfigTemperature,
    ConfigUrl, LlmConfig, Result,
};

#[derive(Clone, Debug)]
pub struct LlmAdvisor {
    client: Client<OpenAIConfig>,
    schemas: Schemas,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    max_completion_tokens: u32,
}

impl LlmAdvisor {
    pub fn new(config: LlmConfig) -> Self {
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
    async fn advise(
        &self,
        reactor_snapshots: impl IntoIterator<Item = ReactorSnapshot>,
    ) -> Result<Advice> {
        let Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
        } = self;

        let messages = {
            let mut messages = Vec::new();
            messages.push(ChatCompletionRequestMessage::Developer(
                ChatCompletionRequestDeveloperMessageArgs::default()
                    .name("god")
                    .content(ChatCompletionRequestDeveloperMessageContent::Text(
                        include_str!("base_prompt.md").to_owned(),
                    ))
                    .build()?,
            ));
            for reactor_snapshot in reactor_snapshots {
                messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .name("reactor-monitor")
                        .content(ChatCompletionRequestUserMessageContent::Text(
                            self.summarise_reactor_snapshot(&reactor_snapshot),
                        ))
                        .build()?,
                ));
            }
            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .name("boss")
                    .content(ChatCompletionRequestUserMessageContent::Text(
                        "what do you recommend?".to_owned(),
                    ))
                    .build()?,
            ));
            messages
        };
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
        let ret = serde_json::from_str(&raw_advice)?;
        Ok(ret)
    }
}

impl LlmAdvisor {
    fn summarise_reactor_snapshot(&self, reactor_snapshot: &ReactorSnapshot) -> String {
        let ReactorSnapshot { timestamp, state } = reactor_snapshot;
        match state {
            ReactorState::Destroyed => formatdoc!(
                "
                    ### Reactor state at {timestamp}

                    Reactor sensors malfunctioned; no data available.
                "
            ),
            ReactorState::Intact(intact_state) => {
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
                } = intact_state;
                let mode = match mode {
                    ReactorMode::Active => "active",
                    ReactorMode::Inactive => "shut down",
                };
                formatdoc!(
                    "
                        ### Reactor state at {timestamp}:

                        - **mode**: {mode}.
                        - **temperature**: {temperature}.
                        - **coolant_filled**: {coolant_filled}.
                        - **heated_coolant_filled**: {heated_coolant_filled}.
                        - **fuel_filled**: {fuel_filled}.
                        - **waste_filled**: {waste_filled}.
                        - **actual_burn_rate**: {actual_burn_rate}.
                        - **target_burn_rate**: {target_burn_rate}.
                        - **damage_percent**: {damage_percent}.
                        - **heating_rate**: {heating_rate}.
                        - **boil_efficiency**: {boil_efficiency}.
                    "
                )
            }
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
