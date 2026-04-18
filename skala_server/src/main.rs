use std::fs;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use log::{error, info};
use skala_server::advisor::LlmAdvisor;
use skala_server::{App, Config, GeneralConfig, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    colog::init();
    if let Err(err) = run().await {
        error!("{err:?}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let Args { config, db_path } = args;
    let config = read_config(config)?.unwrap_or_default();
    let Config {
        general: general_config,
        llm: llm_config,
    } = config;
    let GeneralConfig {
        port,
        reactor_snapshot_window_limit,
    } = general_config;

    let db_pool = load_db_pool(db_path).await?;
    let advisor = LlmAdvisor::new(llm_config);
    let reactor_snapshot_window_limit = reactor_snapshot_window_limit.into_inner();
    let app = App::new(db_pool.clone(), reactor_snapshot_window_limit, advisor);

    let port = port.into_inner();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.context("cannot bind tcp")?;

    let addr = listener.local_addr().context("Cannot get local addr")?;
    info!("listening on {addr}");

    axum::serve(listener, app.into_router())
        .await
        .context("Cannot serve app")?;
    db_pool.close().await; // Await optimisation.
    Ok(())
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(long, default_value = "skala.toml")]
    config: Utf8PathBuf,

    #[clap(long, default_value = "skala.db")]
    db_path: Utf8PathBuf,
}

fn read_config(path: impl AsRef<Utf8Path>) -> Result<Option<Config>> {
    let path = path.as_ref();
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err)
                .with_context(|| anyhow!("cannot read {path}"))
                .map_err(Into::into);
        }
    };
    let config = toml::from_str(&raw).with_context(|| anyhow!("cannot parse config at {path}"))?;
    Ok(Some(config))
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
