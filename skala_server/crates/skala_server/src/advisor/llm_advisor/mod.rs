pub mod feedback;

use anyhow::Context;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessageArgs,
    ChatCompletionRequestDeveloperMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequestArgs, ReasoningEffort, ResponseFormat, ResponseFormatJsonSchema,
    Verbosity,
};
use indoc::formatdoc;
use log::info;
use schemars::schema_for;
use serde_json::Value;

use crate::advisor::feedback::FeedbackProvider;
use crate::advisor::{Advice, Advisor, PastAction, PastEvent, ReactorSnapshot};
use crate::reactor::ReactorState;
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
    feedback_provider: FeedbackProvider,
}

impl LlmAdvisor {
    pub fn new(config: LlmConfig) -> Self {
        let LlmConfig {
            url: ConfigUrl(url),
            temperature: ConfigTemperature(temperature),
            frequency_penalty: ConfigFrequencyPenalty(frequency_penalty),
            presence_penalty: ConfigPresencePenalty(presence_penalty),
            max_completion_tokens: ConfigMaxCompletionTokens(max_completion_tokens),
            feedback_regime,
            feedback,
        } = config;

        let client_config = OpenAIConfig::new().with_api_base(url);
        let client = Client::with_config(client_config);

        let schemas = Schemas::new();
        let feedback_provider = FeedbackProvider::new(feedback_regime, feedback);
        Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
            feedback_provider,
        }
    }
}

impl Advisor for LlmAdvisor {
    async fn advise(&self, past_events: impl IntoIterator<Item = PastEvent>) -> Result<Advice> {
        let Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
            feedback_provider,
        } = self;

        info!("creating message");
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

            let mut feedback = feedback_provider.feedback();
            for past_event in past_events {
                match past_event {
                    PastEvent::ReactorSnapshot(snapshot) => {
                        let content = self.summarise_reactor_snapshot(&snapshot);
                        let message = ChatCompletionRequestUserMessageArgs::default()
                            .name("reactor-monitor")
                            .content(content)
                            .build()?;
                        messages.push(message.into());
                    }
                    PastEvent::Action(action) => {
                        let content = self.summarise_action(&action);
                        let message = ChatCompletionRequestAssistantMessageArgs::default()
                            .content(content)
                            .build()?;
                        messages.push(message.into());

                        if let Some(feedback_content) = feedback.next() {
                            let feedback_message = ChatCompletionRequestUserMessageArgs::default()
                                .name("boss")
                                .content(feedback_content.to_string())
                                .build()?;
                            messages.push(feedback_message.into());
                        }
                    }
                }
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

        for message in &messages {
            log::info!("{}", serde_json::to_string(message).unwrap());
        }

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

        info!("awaiting llm response");
        let response = client.chat().create(req).await?;
        info!("got llm response");
        let raw_advice = response
            .choices
            .into_iter()
            .next()
            .context("llm returned too few choices")?
            .message
            .content
            .context("llm choice returned no content")?;

        info!("parsing advice '{raw_advice}'");
        let ret = serde_json::from_str(&raw_advice)?;
        Ok(ret)
    }
}

impl LlmAdvisor {
    fn summarise_reactor_snapshot(&self, reactor_snapshot: &ReactorSnapshot) -> String {
        let ReactorSnapshot { timestamp, state } = reactor_snapshot;
        if matches!(state, ReactorState::Destroyed) {
            return formatdoc!(
                "
                    ### Reactor state at {timestamp}

                    Reactor sensors malfunctioned; no data available.
                "
            );
        }

        let state_json = serde_json::to_string(state).unwrap();
        formatdoc!(
            "
                ### Reactor state at {timestamp}

                ```json
                {state_json}
                ```
            "
        )
    }

    fn summarise_action(&self, action: &PastAction) -> String {
        let PastAction {
            timestamp: _,
            action,
        } = action;
        serde_json::to_string(action).unwrap()
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
