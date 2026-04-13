mod advisor;
mod app;
mod reactor;
mod routes;

use std::fs;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::{Context, anyhow};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use log::{error, info};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tokio::net::TcpListener;

use crate::advisor::LlmAdvisor;
use crate::app::App;

type Result<T, E = Error> = anyhow::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct Error(anyhow::Error);

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
impl_from_error!(async_openai::error::OpenAIError);
impl_from_error!(sqlx::Error);

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    colog::init();
    if let Err(err) = run().await {
        error!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let Args { config, db_path } = args;
    let config = Config::try_read(config)?;
    let Config {
        general: general_config,
        llm: llm_config,
    } = config;
    let GeneralConfig { port } = general_config;

    let db_pool = load_db_pool(db_path).await?;
    let advisor = LlmAdvisor::new(llm_config);
    let app = App::new(db_pool, advisor);

    let ConfigPort(port) = port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.context("cannot bind tcp")?;

    let addr = listener.local_addr().context("Cannot get local addr")?;
    info!("listening on {addr}");

    axum::serve(listener, app.into_router())
        .await
        .context("Cannot serve app")?;
    Ok(())
}

async fn load_db_pool(path: impl Into<Utf8PathBuf>) -> Result<SqlitePool> {
    let path = path.into();
    let url = format!("sqlite://{path}");
    let opts = SqliteConnectOptions::from_str(&url)?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .optimize_on_close(true, None);
    let ret = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await?;
    Ok(ret)
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(long, default_value = "skala.toml")]
    config: Utf8PathBuf,

    #[clap(long, default_value = "skala.db")]
    db_path: Utf8PathBuf,
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(rename = "skala", default)]
    general: GeneralConfig,
    #[serde(default)]
    llm: LlmConfig,
}

impl Config {
    fn try_read(path: impl AsRef<Utf8Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).with_context(|| anyhow!("cannot read {path}"))?;
        Ok(toml::from_str(&raw).with_context(|| anyhow!("cannot parse config at {path}"))?)
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename = "kebab-case")]
struct GeneralConfig {
    #[serde(default)]
    port: ConfigPort,
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
struct ConfigPort(u16);

impl Default for ConfigPort {
    fn default() -> Self {
        Self(15000)
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename = "kebab-case")]
struct LlmConfig {
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
