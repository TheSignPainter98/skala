use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::time::sleep;

use crate::app::{AppAction, AppState, StartupSelection, ensure_reactor_selection, handle_key};

pub mod app;
pub mod data;
pub mod ui;

const FRAME_DURATION: Duration = Duration::from_millis(33);
const WATCH_RELOAD_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphOptions {
    pub db_path: PathBuf,
    pub reactor: Option<String>,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("skala.db"),
            reactor: None,
        }
    }
}

pub async fn run(options: &GraphOptions) -> Result<()> {
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    install_signal_handler(&shutdown_requested)?;
    wait_for_reactor_at_startup(
        &options.db_path,
        options.reactor.as_deref(),
        &shutdown_requested,
    )
    .await?;
    let mut app = AppState::load(&options.db_path, options.reactor.as_deref()).await?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_app(&mut terminal, &mut app, &shutdown_requested).await;
    let close_result = app.close_database().await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result?;
    close_result
}

async fn wait_for_reactor_at_startup(
    path: &Path,
    requested_reactor: Option<&str>,
    shutdown_requested: &AtomicBool,
) -> Result<()> {
    let mut announced_wait = false;

    loop {
        if shutdown_requested.load(Ordering::Relaxed) {
            return Ok(());
        }

        match ensure_reactor_selection(path, requested_reactor).await? {
            StartupSelection::Ready => return Ok(()),
            StartupSelection::WaitForReactor => {
                if !announced_wait {
                    print_waiting_message(path, requested_reactor);
                    announced_wait = true;
                }
                sleep(WATCH_RELOAD_INTERVAL).await;
            }
        }
    }
}

fn print_waiting_message(path: &Path, requested_reactor: Option<&str>) {
    match requested_reactor {
        Some(reactor) => eprintln!(
            "No reactors with events are available in {} yet; waiting for reactor `{reactor}`.",
            path.display()
        ),
        None => eprintln!(
            "No reactors with events are available in {} yet; waiting for data.",
            path.display()
        ),
    }
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    shutdown_requested: &AtomicBool,
) -> Result<()> {
    let mut last_watch_reload = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if shutdown_requested.load(Ordering::Relaxed) {
            return Ok(());
        }

        if last_watch_reload.elapsed() >= WATCH_RELOAD_INTERVAL {
            if let Err(error) = app.reload().await {
                app.set_watch_reload_error(&error.to_string());
            }
            last_watch_reload = Instant::now();
        }

        if event::poll(FRAME_DURATION)?
            && let Event::Key(key) = event::read()?
        {
            if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                continue;
            }

            if let AppAction::Quit = handle_key(app, key).await? {
                return Ok(());
            }
        }
    }
}

fn install_signal_handler(shutdown_requested: &Arc<AtomicBool>) -> Result<()> {
    let shutdown_requested = Arc::clone(shutdown_requested);
    ctrlc::set_handler(move || {
        shutdown_requested.store(true, Ordering::Relaxed);
    })?;
    Ok(())
}
