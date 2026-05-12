use std::io::{self, stdout};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use skala_graph::app::{
    AppAction, AppState, StartupSelection, ensure_reactor_selection, handle_key,
};
use skala_graph::ui;

const FRAME_DURATION: Duration = Duration::from_millis(33);
const WATCH_RELOAD_INTERVAL: Duration = Duration::from_millis(250);

fn main() -> Result<()> {
    let options = CliOptions::parse();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    install_signal_handler(&shutdown_requested)?;
    wait_for_reactor_at_startup(
        &options.db_path,
        options.reactor.as_deref(),
        &shutdown_requested,
        options.watch,
    )?;
    let mut app = AppState::load(&options.db_path, options.reactor.as_deref())?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_app(&mut terminal, &mut app, &shutdown_requested, options.watch);
    let close_result = app.close_database();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result?;
    close_result
}

fn wait_for_reactor_at_startup(
    path: &Path,
    requested_reactor: Option<&str>,
    shutdown_requested: &AtomicBool,
    watch_enabled: bool,
) -> Result<()> {
    let mut announced_wait = false;

    loop {
        if shutdown_requested.load(Ordering::Relaxed) {
            return Ok(());
        }

        match ensure_reactor_selection(path, requested_reactor)? {
            StartupSelection::Ready => return Ok(()),
            StartupSelection::WaitForReactor if watch_enabled => {
                if !announced_wait {
                    print_waiting_message(path, requested_reactor);
                    announced_wait = true;
                }
                sleep(WATCH_RELOAD_INTERVAL);
            }
            StartupSelection::WaitForReactor => {
                bail!("the database does not contain any reactors with events");
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    shutdown_requested: &AtomicBool,
    watch_enabled: bool,
) -> Result<()> {
    let mut last_watch_reload = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(frame, app, watch_enabled))?;

        if shutdown_requested.load(Ordering::Relaxed) {
            return Ok(());
        }

        if watch_enabled && last_watch_reload.elapsed() >= WATCH_RELOAD_INTERVAL {
            if let Err(error) = app.reload() {
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

            if let AppAction::Quit = handle_key(app, key)? {
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

#[derive(Debug, Eq, Parser, PartialEq)]
#[command(
    name = "skala-graph",
    about = "Open a SQLite database and render reactor metrics in a terminal graph.",
    long_about = "Open a SKALA SQLite database and render reactor metrics in a terminal graph.\n\nIf the database contains one reactor, it is selected automatically. Use --reactor to preselect a reactor by name when the database contains more than one."
)]
struct CliOptions {
    #[arg(
        value_name = "DB_PATH",
        help = "Path to the SQLite database file to inspect",
        default_value = "skala.db"
    )]
    db_path: PathBuf,
    #[arg(
        long,
        value_name = "NAME",
        help = "Preselect a reactor by name before opening the interface"
    )]
    reactor: Option<String>,
    #[arg(
        long,
        help = "Reload the database from disk every 0.25 seconds and wait at startup if no reactors have events yet"
    )]
    watch: bool,
}

#[cfg(test)]
mod tests {
    use super::CliOptions;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn cli_shape_matches_expected_values() {
        let options = CliOptions {
            db_path: PathBuf::from("sample.db"),
            reactor: Some("reactor_53".to_owned()),
            watch: true,
        };

        assert_eq!(options.db_path, PathBuf::from("sample.db"));
        assert_eq!(options.reactor.as_deref(), Some("reactor_53"));
        assert!(options.watch);
    }

    #[test]
    fn clap_parses_expected_arguments() {
        let options = CliOptions::parse_from([
            "skala-graph",
            "sample.db",
            "--reactor",
            "reactor_53",
            "--watch",
        ]);

        assert_eq!(options.db_path, PathBuf::from("sample.db"));
        assert_eq!(options.reactor.as_deref(), Some("reactor_53"));
        assert!(options.watch);
    }
}
