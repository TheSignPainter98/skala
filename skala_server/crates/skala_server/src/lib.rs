pub mod advisor;
mod app;
mod components;
mod middleware;
mod routes;
mod time;

use axum::http::StatusCode;
use axum::response::IntoResponse;

pub use crate::advisor::feedback::Feedback;
pub use crate::app::App;
pub use crate::components::reactor::{
    ActualBurnRate, IntactReactorSnapshot, MaxBurnRate, ReactorMode, ReactorSnapshot,
    TargetBurnRate,
};
pub use crate::components::turbine::{IntactTurbineSnapshot, TurbineSnapshot};

pub type Result<T, E = Error> = anyhow::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(anyhow::Error);

macro_rules! impl_from_error {
    ($error_type:path) => {
        impl From<$error_type> for Error {
            fn from(t: $error_type) -> Self {
                Self(t.into())
            }
        }
    };
}
impl_from_error!(anyhow::Error);
impl_from_error!(arboard::Error);
impl_from_error!(async_openai::error::OpenAIError);
impl_from_error!(notify_rust::error::Error);
impl_from_error!(serde_json::Error);
impl_from_error!(sqlx::Error);
impl_from_error!(sqlx::migrate::MigrateError);

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let repr = self.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, repr).into_response()
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "skala", default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub llm: LlmConfig,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    #[serde(default)]
    pub port: ConfigPort,

    #[serde(default)]
    pub advisor_kind: AdvisorKind,

    #[serde(default)]
    pub snapshot_window_limit: ReactorSnapshotWindowLimit,
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
pub struct ConfigPort(u16);

impl ConfigPort {
    pub fn into_inner(self) -> u16 {
        let Self(port) = self;
        port
    }
}

impl Default for ConfigPort {
    fn default() -> Self {
        Self(15000)
    }
}

#[derive(Copy, Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorKind {
    CopyPaste,
    #[default]
    OpenAi,
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
pub struct ReactorSnapshotWindowLimit(u16);

impl ReactorSnapshotWindowLimit {
    pub fn into_inner(self) -> u16 {
        let Self(limit) = self;
        limit
    }
}

impl Default for ReactorSnapshotWindowLimit {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    #[serde(default)]
    url: ConfigUrl,
    #[serde(default)]
    temperature: ConfigTemperature,
    #[serde(default)]
    frequency_penalty: ConfigFrequencyPenalty,
    #[serde(default)]
    presence_penalty: ConfigPresencePenalty,
    #[serde(default)]
    max_completion_tokens: ConfigMaxCompletionTokens,

    #[serde(default)]
    feedback_regime: FeedbackRegime,
    #[serde(default)]
    feedback: FeedbackConfig,
}

#[derive(Debug, serde::Deserialize)]
struct ConfigUrl(String);

impl Default for ConfigUrl {
    fn default() -> Self {
        Self("http://localhost:8326".to_owned())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConfigTemperature(f32);

impl Default for ConfigTemperature {
    fn default() -> Self {
        Self(1.5)
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConfigFrequencyPenalty(f32);

impl Default for ConfigFrequencyPenalty {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConfigPresencePenalty(f32);

impl Default for ConfigPresencePenalty {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConfigMaxCompletionTokens(u32);

impl Default for ConfigMaxCompletionTokens {
    fn default() -> Self {
        Self(512)
    }
}

#[derive(Copy, Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FeedbackRegime {
    #[default]
    Absent,
    Positive,
    Negative,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
struct FeedbackConfig {
    #[serde(default)]
    positive: Vec<Feedback>,
    #[serde(default)]
    negative: Vec<Feedback>,
}
