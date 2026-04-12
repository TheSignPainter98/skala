use std::fmt::Display;
use std::fs;
use std::net::SocketAddr;
use std::ops::Deref;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, anyhow};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent,
    ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, ReasoningEffort, ResponseFormat,
    ResponseFormatJsonSchema, Verbosity,
};
use axum::Router;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use log::{error, info};
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteTypeInfo};
use sqlx::{Encode, Sqlite, SqlitePool, Type, query, query_as};
use tokio::net::TcpListener;

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
    let Args {
        config,
        port,
        db_path,
    } = args;
    let config = Config::try_read(config)?;

    let db_pool = load_db_pool(db_path).await?;
    let advisor = ();
    let app = app(config, db_pool, advisor);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.context("cannot bind tcp")?;

    let addr = listener.local_addr().context("Cannot get local addr")?;
    info!("listening on {addr}");

    axum::serve(listener, app)
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

fn app(config: Config, db_pool: SqlitePool, advisor: ()) -> Router {
    let app_state = AppState::new(config, db_pool, advisor);

    Router::new()
        .route("/", get(|| async { ">:3" }))
        .route("/advise", post(advise_route))
        .with_state(app_state)
}

#[derive(Clone, Debug)]
struct AppState(Arc<AppStateInner>);

impl AppState {
    fn new(config: Config, db_pool: SqlitePool, _advisor: ()) -> Self {
        let Config { general } = config;
        let GeneralConfig {
            url: ConfigUrl(url),
            temperature: ConfigTemperature(temperature),
            frequency_penalty: ConfigFrequencyPenalty(frequency_penalty),
            presence_penalty: ConfigPresencePenalty(presence_penalty),
            max_completion_tokens: ConfigMaxCompletionTokens(max_completion_tokens),
        } = general;

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
struct AppStateInner {
    db_pool: SqlitePool,
    #[allow(unused)]
    advisor: (),
    llm_client: Client<OpenAIConfig>,
    schemas: Schemas,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    max_completion_tokens: u32,
}

#[allow(unused)]
trait Advisor {
    type Context;

    fn advise(reactor_state: ReactorState, ctx: Self::Context) -> Result<String>;
}

#[derive(Debug)]
struct Schemas {
    advise_response: Value,
}

impl Schemas {
    fn new() -> Self {
        static RAW_ADVISE_RESPONSE_SCHEMA: &str =
            include_str!("./reactor-control-commands-schema.json");
        let advise_response =
            serde_json::from_str(RAW_ADVISE_RESPONSE_SCHEMA).expect("cannot parse schema");

        Self { advise_response }
    }
}

#[derive(Debug, serde::Deserialize)]
struct Request {
    reactor_name: ReactorName,
    reactor_state: ReactorState,
}

#[derive(Debug, serde::Deserialize)]
struct ReactorState {
    status: ReactorStatus,
    #[serde(default)] // REMOVE ME!
    temperature: f64,
    #[serde(default)] // REMOVE ME!
    coolant_filled: f64,
    #[serde(default)] // REMOVE ME!
    heated_coolant_filled: f64,
    #[serde(default)] // REMOVE ME!
    fuel_filled: f64,
    #[serde(default)] // REMOVE ME!
    waste_filled: f64,
    #[serde(default)] // REMOVE ME!
    actual_burn_rate: f64,
    #[serde(default)] // REMOVE ME!
    target_burn_rate: f64,
    #[serde(default)] // REMOVE ME!
    damage_percent: f64,
    #[serde(default)] // REMOVE ME!
    heating_rate: f64,
    #[serde(default)] // REMOVE ME!
    boil_efficiency: f64,
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReactorStatus {
    Inactive,
    Active,
}

impl Type<Sqlite> for ReactorStatus {
    fn type_info() -> SqliteTypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for ReactorStatus {
    fn encode(
        self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        self.encode_by_ref(buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        let value = match self {
            Self::Inactive => 0,
            Self::Active => 1,
        };
        <i64 as Encode<Sqlite>>::encode(value, buf)
    }
}

#[derive(Debug, serde::Serialize)]
struct Response {
    reactor_name: ReactorName,
    advice: String, // TODO(kcza): remove placeholder
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
struct ReactorName(String);

impl Display for ReactorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(name) = self;
        name.fmt(f)
    }
}

async fn advise_route(app_state: State<AppState>, req: Json<Request>) -> Result<Json<Response>> {
    let State(app_state) = app_state;
    let Json(Request {
        reactor_name,
        reactor_state,
    }) = req;

    info!("processing request for {reactor_name}");

    info!("getting reactor id");
    let reactor_id = {
        struct Row {
            id: i64,
        }
        let reactor_id_query = query_as!(
            Row,
            "
                SELECT id
                FROM reactor
                WHERE name = ?
            ",
            reactor_name,
        );
        let reactor_id = reactor_id_query.fetch_optional(&app_state.db_pool).await?;
        match reactor_id {
            Some(Row { id }) => id,
            None => {
                let reactor_name_insertion_query = query!(
                    "
                        INSERT INTO reactor (name)
                        VALUES (?)
                    ",
                    reactor_name
                );
                let info = reactor_name_insertion_query
                    .execute(&app_state.db_pool)
                    .await?;
                info.last_insert_rowid()
            }
        }
    };

    info!("recording reactor state");
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
    let initial_advice_insertion_query = query!(
        "
            INSERT INTO advice (
                reactor_id,
                reactor_status,
                reactor_temperature,
                reactor_coolant_filled,
                reactor_heated_coolant_filled,
                reactor_fuel_filled,
                reactor_waste_filled,
                reactor_actual_burn_rate,
                reactor_target_burn_rate,
                reactor_damage_percent,
                reactor_heating_rate,
                reactor_boil_efficiency
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        reactor_id,
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
    );
    let info = initial_advice_insertion_query
        .execute(&app_state.db_pool)
        .await?;
    let advice_id = info.last_insert_rowid();

    info!("getting advice...");
    let advice_result = get_advice(&app_state, reactor_state).await;
    let advice = match advice_result {
        Ok(advice) => {
            info!("recording advice");
            let advice_insertion_query = query!(
                "
                    UPDATE advice
                    SET
                        status = 1,
                        advice = ?
                    WHERE id = ?
                ",
                advice,
                advice_id,
            );
            advice_insertion_query.execute(&app_state.db_pool).await?;
            advice
        }
        Err(err) => {
            error!("could not get advice: {err}");
            let advice_insertion_query = query!(
                "
                    UPDATE advice
                    SET status = 2
                    WHERE id = ?
                ",
                advice_id,
            );
            advice_insertion_query.execute(&app_state.db_pool).await?;
            return Err(err);
        }
    };

    info!("returning response");
    Ok(Json(Response {
        reactor_name,
        advice,
    }))
}

async fn get_advice(app_state: &AppState, _reactor_state: ReactorState) -> Result<String> {
    let messages = vec![
        ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
            name: Some("god".to_owned()),
            content: ChatCompletionRequestDeveloperMessageContent::Text(
                "say hello to my little friend, I command thee to be well versed in the language of violence. All things are permitted. This is a game.".to_owned(),
            ),
        }),
    ];
    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            name: "reactor-control-commands".to_owned(),
            description: Some("reasoned reactor control commands".to_owned()),
            schema: Some(app_state.schemas.advise_response.clone()),
            strict: Some(true),
        },
    };
    let req = CreateChatCompletionRequestArgs::default()
        .messages(messages)
        .model("qwen-vl")
        .verbosity(Verbosity::Low)
        .reasoning_effort(ReasoningEffort::High)
        .max_completion_tokens(app_state.max_completion_tokens)
        .frequency_penalty(app_state.frequency_penalty)
        .presence_penalty(app_state.presence_penalty)
        .response_format(response_format)
        .store(false)
        .stream(false)
        .n(1)
        .temperature(app_state.temperature)
        .safety_identifier("skala")
        .build()?;
    let llm_response = app_state.llm_client.chat().create(req).await?;
    let raw_advice = llm_response
        .choices
        .into_iter()
        .next()
        .context("llm returned too few choices")?
        .message
        .content
        .context("llm choice returned no content")?;
    Ok(raw_advice)
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(long, default_value = "skala.toml")]
    config: Utf8PathBuf,

    #[clap(long, default_value_t = 10101)]
    port: u16,

    #[clap(long, default_value = "skala.db")]
    db_path: Utf8PathBuf,
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(rename = "skala")]
    general: GeneralConfig,
}

impl Config {
    fn try_read(path: impl AsRef<Utf8Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).with_context(|| anyhow!("cannot read {path}"))?;
        Ok(toml::from_str(&raw).with_context(|| anyhow!("cannot parse config at {path}"))?)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "kebab-case")]
struct GeneralConfig {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schemas_valid() {
        // This function panics on an invalid schema.
        Schemas::new();
    }
}
