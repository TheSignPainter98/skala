use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use skala_graph::app::{AppAction, AppState, handle_key};
use skala_graph::ui;

const FRAME_DURATION: Duration = Duration::from_millis(33);

fn main() -> Result<()> {
    let options = CliOptions::parse();
    let mut app = AppState::load(&options.db_path, options.reactor.as_deref())?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

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

#[derive(Debug, Eq, Parser, PartialEq)]
#[command(
    name = "skala-graph",
    about = "Open a SQLite database and render reactor metrics in a terminal graph."
)]
struct CliOptions {
    #[arg(value_name = "db_path")]
    db_path: PathBuf,
    #[arg(long)]
    reactor: Option<String>,
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
        };

        assert_eq!(options.db_path, PathBuf::from("sample.db"));
        assert_eq!(options.reactor.as_deref(), Some("reactor_53"));
    }

    #[test]
    fn clap_parses_expected_arguments() {
        let options =
            CliOptions::parse_from(["skala-graph", "sample.db", "--reactor", "reactor_53"]);

        assert_eq!(options.db_path, PathBuf::from("sample.db"));
        assert_eq!(options.reactor.as_deref(), Some("reactor_53"));
    }
}
