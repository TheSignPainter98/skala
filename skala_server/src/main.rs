use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, ErrorKind, Write};
use std::net::SocketAddr;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use indoc::indoc;
use indoc::writedoc;
use log::{error, info};
use quicktype::QuicktypeDerivedType;
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
    let Args { command } = Args::parse();
    match command {
        Command::Init { dir } => run_init(dir).await,
        Command::Run { config, db_path } => run_run(config, db_path).await,
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
        const MANIFEST_CONTENT: &str = indoc! {r#"
            [skala]
            port = 15000
            reactor-snapshot-window-limit = 25

            [llm]
            url = "http://localhost:8326"
            temperature = 1.5
            frequency-penalty = 1.0
            presence-penalty = 1.0
            max-completion-tokens = 512
        "#};
        let mut writer = BufWriter::new(txn.create(&manifest_path, false)?);
        writer
            .write_all(MANIFEST_CONTENT.as_bytes())
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
///
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

async fn run_run(config: Utf8PathBuf, db_path: Utf8PathBuf) -> Result<()> {
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
    Run {
        #[clap(long, default_value = "skala.toml")]
        config: Utf8PathBuf,

        #[clap(long, default_value = "skala.db")]
        db_path: Utf8PathBuf,
    },

    /// Show quicktype specs
    PrintQuicktypeSpecs,
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
                declare_type '{name}', [==========[
                    {spec}
                ]==========]
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
