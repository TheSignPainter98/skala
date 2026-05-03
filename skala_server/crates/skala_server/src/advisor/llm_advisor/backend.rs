use std::fmt::Debug;
use std::io::{self, BufRead};
use std::sync::Arc;

use anyhow::{Context, anyhow};
use arboard::Clipboard;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ReasoningEffort, ResponseFormat, ResponseFormatJsonSchema,
        Verbosity,
    },
};
use futures_util::lock::Mutex;
use log::{info, warn};
use serde::Deserialize;

use crate::{
    ConfigFrequencyPenalty, ConfigMaxCompletionTokens, ConfigPresencePenalty, ConfigTemperature,
    ConfigUrl, Error, LlmConfig, Result,
    advisor::llm_advisor::{PromptInfo, Schemas},
};

#[derive(Clone, Debug)]
pub enum Backend {
    CopyPaste(CopyPasteBackend),
    OpenAi(Box<OpenAiBackend>),
}

impl Backend {
    pub(crate) async fn fetch<T: for<'de> Deserialize<'de>>(
        &self,
        prompt_info: impl IntoIterator<Item = PromptInfo<'_>> + Send,
    ) -> Result<T> {
        match self {
            Self::CopyPaste(inner) => inner.fetch(prompt_info).await,
            Self::OpenAi(inner) => inner.fetch(prompt_info).await,
        }
    }
}

#[derive(Clone)]
pub struct CopyPasteBackend {
    // NOTE: This really shouldn't be an Arc-Mutex, but cloning is required elsewhere
    // and this works well enough.
    // TODO(kcza): fis this.
    clipboard: Arc<Mutex<Clipboard>>,
}

impl CopyPasteBackend {
    pub fn new() -> Result<Self> {
        let clipboard = Arc::new(Mutex::new(Clipboard::new()?));
        Ok(Self { clipboard })
    }

    pub(crate) async fn fetch<T: for<'de> Deserialize<'de>>(
        &self,
        prompt_info: impl IntoIterator<Item = PromptInfo<'_>> + Send,
    ) -> Result<T> {
        let prompt = prompt_info
            .into_iter()
            .map(|info| info.summary())
            .collect::<Vec<_>>()
            .join(", ");

        self.clipboard.lock().await.set_text(prompt)?;
        let response = {
            let mut stdin = io::stdin().lock();
            let mut buf = String::new();
            let mut remaining_attempts = 10;
            loop {
                if remaining_attempts == 0 {
                    return Err(anyhow!("too many attempts").into());
                }

                buf.clear();
                eprint!("Prompt copied to clipboard, please paste 1-line response> ");
                stdin
                    .read_line(&mut buf)
                    .with_context(|| anyhow!("cannot read line to buffer"))?;
                eprintln!();

                match serde_json::from_str(&buf) {
                    Ok(response) => break response,
                    Err(err) => {
                        warn!("{err}");
                        warn!(
                            "prompt ignored, please re-enter (attempts remaining {remaining_attempts})"
                        );
                        remaining_attempts -= 1;
                    }
                }
            }
        };
        Ok(response)
    }
}

impl From<CopyPasteBackend> for Backend {
    fn from(inner: CopyPasteBackend) -> Self {
        Self::CopyPaste(inner)
    }
}

impl Debug for CopyPasteBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopyPasteBackend")
            .field("clipboard", &"_")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct OpenAiBackend {
    client: Client<OpenAIConfig>,
    schemas: Schemas,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    max_completion_tokens: u32,
}

impl OpenAiBackend {
    pub fn new(config: &LlmConfig) -> Self {
        let LlmConfig {
            url: ConfigUrl(url),
            temperature: ConfigTemperature(temperature),
            frequency_penalty: ConfigFrequencyPenalty(frequency_penalty),
            presence_penalty: ConfigPresencePenalty(presence_penalty),
            max_completion_tokens: ConfigMaxCompletionTokens(max_completion_tokens),
            feedback_regime: _,
            feedback: _,
        } = config;
        let client_config = OpenAIConfig::new().with_api_base(url);
        let client = Client::with_config(client_config);
        let schemas = Schemas::new();
        Self {
            client,
            schemas,
            temperature: *temperature,
            frequency_penalty: *frequency_penalty,
            presence_penalty: *presence_penalty,
            max_completion_tokens: *max_completion_tokens,
        }
    }

    pub(crate) async fn fetch<T: for<'de> Deserialize<'de>>(
        &self,
        prompt_info: impl IntoIterator<Item = PromptInfo<'_>> + Send,
    ) -> Result<T> {
        let Self {
            client,
            schemas,
            temperature,
            frequency_penalty,
            presence_penalty,
            max_completion_tokens,
        } = self;

        let messages: Vec<_> = prompt_info
            .into_iter()
            .map(ChatCompletionRequestMessage::try_from)
            .collect::<Result<_>>()?;
        log::info!(
            "Formatted messages as: {:?}",
            serde_json::to_string(&messages).unwrap()
        );

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

impl TryFrom<PromptInfo<'_>> for ChatCompletionRequestMessage {
    type Error = Error;

    fn try_from(info: PromptInfo<'_>) -> std::result::Result<Self, Self::Error> {
        let summary = info.summary();
        match info {
            PromptInfo::BasePrompt(_) => {
                let ret = ChatCompletionRequestDeveloperMessageArgs::default()
                    .name("god")
                    .content(summary)
                    .build()?
                    .into();
                Ok(ret)
            }
            PromptInfo::Snapshot(_) => {
                let ret = ChatCompletionRequestUserMessageArgs::default()
                    .name("reactor-monitor")
                    .content(summary)
                    .build()?
                    .into();
                Ok(ret)
            }
            PromptInfo::PastAction(_) => {
                let ret = ChatCompletionRequestAssistantMessageArgs::default()
                    .content(summary)
                    .build()?
                    .into();
                Ok(ret)
            }
            PromptInfo::Feedback(_) => {
                let ret = ChatCompletionRequestUserMessageArgs::default()
                    .name("boss")
                    .content(summary)
                    .build()?
                    .into();
                Ok(ret)
            }
            PromptInfo::TargetEnergyProductionRate(_) => {
                let ret = ChatCompletionRequestUserMessageArgs::default()
                    .name("boss")
                    .content(summary)
                    .build()?
                    .into();
                Ok(ret)
            }
        }
    }
}

impl From<OpenAiBackend> for Backend {
    fn from(inner: OpenAiBackend) -> Self {
        Self::OpenAi(Box::new(inner))
    }
}
