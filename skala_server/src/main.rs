use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, ErrorKind, Write};
use std::net::SocketAddr;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args as ClapArgs, Parser};
use indoc::writedoc;
use log::{error, info};
use quicktype::QuicktypeDerivedType;
use skala_server::advisor::{Backend, CopyPasteBackend, LlmAdvisor, OpenAiBackend};
use skala_server::{AdvisorKind, App, Config, GeneralConfig, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tokio::net::TcpListener;
use tokio::signal;

const INITIAL_MANIFEST_CONTENT: &str = include_str!("./default_manifest_content.toml");

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
    let Args { command } = Args::parse();
    match command {
        Command::Init { dir } => run_init(dir).await,
        Command::Serve { config, db_path } => run_serve(config, db_path).await,
        Command::Graph(args) => run_graph(args).await,
        Command::PrintQuicktypeSpecs => {
            run_print_quicktype_specs();
            Ok(())
        }
    }
}

async fn run_init(dir: Utf8PathBuf) -> Result<()> {
    let manifest_path = dir.join("skala.toml");
    let database_path = dir.join("skala.db");

    let mut txn = FsTransaction::new();
    if !dir.exists() {
        txn.create_dir_all(dir)?;
    }

    {
        let mut writer = BufWriter::new(txn.create(&manifest_path, false)?);
        writer
            .write_all(INITIAL_MANIFEST_CONTENT.as_bytes())
            .context("cannot write manifest")?;
        info!("created {manifest_path}");
    }

    {
        txn.declare_new_path_created(&database_path)?;
        let connect_options =
            SqliteConnectOptions::from_str(&format!("{database_path}"))?.create_if_missing(true);
        let db_pool = SqlitePool::connect_with(connect_options).await?;
        sqlx::migrate!("./migrations").run(&db_pool).await?;
        info!("created {database_path}");
    }

    txn.commit();
    Ok(())
}

/// A best-effort handler for writing multiple files atomically.
struct FsTransaction {
    committed: bool,
    created_files: BTreeSet<Utf8PathBuf>,
}

impl FsTransaction {
    fn new() -> Self {
        Self {
            committed: false,
            created_files: BTreeSet::new(),
        }
    }

    fn create_dir_all(&mut self, path: impl Into<Utf8PathBuf>) -> Result<()> {
        let path = path.into();
        info!("creating directory {path}");
        self.declare_path_created(&path);
        fs::create_dir_all(&path).with_context(|| anyhow!("cannot create {path}"))?;
        Ok(())
    }

    fn create(&mut self, path: impl Into<Utf8PathBuf>, force: bool) -> Result<File> {
        let path = path.into();
        let ret = fs::OpenOptions::new()
            .create_new(!force)
            .write(true)
            .open(&path)
            .with_context(|| anyhow!("cannot create {path}"))?;
        self.declare_path_created(&path);
        Ok(ret)
    }

    fn declare_new_path_created(&mut self, path: impl Into<Utf8PathBuf>) -> Result<()> {
        let path = path.into();
        if path.exists() {
            return Err(anyhow!("cannot create {path}").into());
        }
        self.declare_path_created(path);
        Ok(())
    }

    fn declare_path_created(&mut self, path: impl Into<Utf8PathBuf>) {
        self.created_files.insert(path.into());
    }

    fn commit(mut self) {
        self.committed = true
    }
}

impl Drop for FsTransaction {
    fn drop(&mut self) {
        let Self {
            committed,
            created_files: created_paths,
        } = self;
        if *committed {
            return;
        }
        let removal_priority = |path: &Utf8Path| {
            path.metadata()
                .ok()
                .map(|metadata| {
                    if metadata.is_symlink() {
                        0
                    } else if metadata.is_file() {
                        1
                    } else if metadata.is_dir() {
                        2
                    } else {
                        unreachable!();
                    }
                })
                .unwrap_or(1)
        };
        let mut created_paths: Vec<_> = created_paths.iter().collect();
        created_paths.sort_by_cached_key(|path| removal_priority(path));
        for path in created_paths {
            let res = if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            };
            if let Err(err) = res {
                error!("cannot remove {path}: {err}");
            }
        }
    }
}

async fn run_serve(config: Utf8PathBuf, db_path: Utf8PathBuf) -> Result<()> {
    let config = read_config(config)?.unwrap_or_default();
    let Config {
        general: general_config,
        llm: llm_config,
    } = config;
    let GeneralConfig {
        port,
        advisor_kind,
        snapshot_window_limit,
    } = general_config;

    let db_pool = load_db_pool(db_path).await?;
    let backend = match advisor_kind {
        AdvisorKind::CopyPaste => Backend::from(CopyPasteBackend::new()?),
        AdvisorKind::OpenAi => Backend::from(OpenAiBackend::new(&llm_config)),
    };
    let advisor = LlmAdvisor::new(llm_config, backend);
    let snapshot_window_limit = snapshot_window_limit.into_inner();
    let app = App::new(db_pool.clone(), snapshot_window_limit, advisor);

    let port = port.into_inner();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.context("cannot bind tcp")?;

    let addr = listener.local_addr().context("Cannot get local addr")?;
    info!("listening on {addr}");

    axum::serve(listener, app.into_router())
        .with_graceful_shutdown(detect_sigint())
        .await
        .context("Cannot serve app")?;
    db_pool.close().await; // Await optimisation.
    Ok(())
}

#[derive(Debug, clap::Parser)]
#[clap(version)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Initialise the database
    Init {
        #[clap(default_value = ".")]
        dir: Utf8PathBuf,
    },

    /// Run the server
    Serve {
        #[clap(long, default_value = "skala.toml")]
        config: Utf8PathBuf,

        #[clap(long, default_value = "skala.db")]
        db_path: Utf8PathBuf,
    },

    /// Open a SQLite database and render reactor metrics in a terminal graph
    Graph(GraphArgs),

    /// Show quicktype specs
    #[clap(hide = true)]
    PrintQuicktypeSpecs,
}

#[derive(ClapArgs, Debug, Eq, PartialEq)]
struct GraphArgs {
    #[arg(
        value_name = "DB_PATH",
        help = "Path to the SQLite database file to inspect",
        default_value = "skala.db"
    )]
    db_path: Utf8PathBuf,

    #[arg(
        long,
        value_name = "NAME",
        help = "Preselect a reactor by name before opening the interface"
    )]
    reactor: Option<String>,
}

async fn run_graph(args: GraphArgs) -> Result<()> {
    #[cfg(feature = "graph")]
    {
        let options = skala_graph::GraphOptions {
            db_path: args.db_path.into_std_path_buf(),
            reactor: args.reactor,
        };
        skala_graph::run(&options).await?;
        Ok(())
    }

    #[cfg(not(feature = "graph"))]
    {
        let _ = args;
        Err(anyhow!(
            "graph support was compiled out at build time; rebuild with default features or enable the `graph` feature"
        )
        .into())
    }
}

fn run_print_quicktype_specs() {
    use std::fmt::Write;

    let mut specs = String::new();
    writeln!(specs, "import 'skala.quicktype' as :declare_type")
        .expect("internal error: buf unwritable");

    for spec in quicktype::derived_type() {
        let QuicktypeDerivedType { name, spec } = spec;
        writeln!(specs).expect("internal error: buf unwritable");
        writedoc!(
            specs,
            "
                declare_type '{name}', [=[{spec}]=]
            ",
        )
        .expect("internal error: buf unwritable");
    }
    print!("{specs}");
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

async fn detect_sigint() {
    let detect_ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("cannot install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("cannot install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = detect_ctrl_c => info!("shutting down"),
        _ = terminate => info!("shutting down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_manifest_content() {
        toml::from_str::<Config>(INITIAL_MANIFEST_CONTENT).unwrap();
    }

    #[test]
    fn cli_parses_init_command_as_before() {
        let args = Args::try_parse_from(["skala", "init", "reactor-dir"]).expect("parse init");

        assert!(matches!(
            args.command,
            Command::Init { dir } if dir == Utf8Path::new("reactor-dir")
        ));
    }

    #[test]
    fn cli_parses_serve_command_as_before() {
        let args = Args::try_parse_from([
            "skala",
            "serve",
            "--config",
            "config.toml",
            "--db-path",
            "state.db",
        ])
        .expect("parse serve");

        assert!(matches!(
            args.command,
            Command::Serve { config, db_path }
                if config == Utf8Path::new("config.toml")
                    && db_path == Utf8Path::new("state.db")
        ));
    }

    #[test]
    fn cli_parses_graph_command_with_defaults() {
        let args = Args::try_parse_from(["skala", "graph"]).expect("parse graph");

        assert!(matches!(
            args.command,
            Command::Graph(GraphArgs { db_path, reactor })
                if db_path == Utf8Path::new("skala.db") && reactor.is_none()
        ));
    }

    #[test]
    fn cli_parses_graph_command_with_database_path() {
        let args = Args::try_parse_from(["skala", "graph", "sample.db"]).expect("parse graph path");

        assert!(matches!(
            args.command,
            Command::Graph(GraphArgs { db_path, reactor })
                if db_path == Utf8Path::new("sample.db") && reactor.is_none()
        ));
    }

    #[test]
    fn cli_parses_graph_command_with_reactor() {
        let args = Args::try_parse_from(["skala", "graph", "sample.db", "--reactor", "reactor_53"])
            .expect("parse graph reactor");

        assert!(matches!(
            args.command,
            Command::Graph(GraphArgs { db_path, reactor })
                if db_path == Utf8Path::new("sample.db")
                    && reactor.as_deref() == Some("reactor_53")
        ));
    }

    #[test]
    fn cli_rejects_unknown_graph_only_flags() {
        let result = Args::try_parse_from(["skala", "graph", "sample.db", "--watch"]);

        assert!(result.is_err());
    }
}
