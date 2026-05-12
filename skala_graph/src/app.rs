use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rusqlite::Connection;

use crate::data::{
    MetricKey, MetricSeries, ReactorData, ReactorSummary, build_series, load_reactor_data,
    load_reactors, open_database, select_reactor, validate_schema,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusPane {
    Reactors,
    Metrics,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChartScaleMode {
    Normalised,
    Raw,
}

impl ChartScaleMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Normalised => Self::Raw,
            Self::Raw => Self::Normalised,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normalised => "normalised",
            Self::Raw => "raw",
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub db_path: PathBuf,
    pub reactors: Vec<ReactorSummary>,
    pub reactor_index: usize,
    pub metric_index: usize,
    pub selected_metrics: BTreeSet<MetricKey>,
    pub chart_scale_mode: ChartScaleMode,
    pub focus: FocusPane,
    pub current_data: ReactorData,
    pub status: String,
    connection: Option<Connection>,
}

impl AppState {
    pub fn load(path: &Path, requested_reactor: Option<&str>) -> Result<Self> {
        let connection = open_database(path)?;
        validate_schema(&connection)?;
        let reactors = load_reactors(&connection)?;
        let reactor_index = select_reactor(&reactors, requested_reactor)?;
        let current_data = load_reactor_data(&connection, reactors[reactor_index].clone())?;

        Ok(Self {
            db_path: path.to_path_buf(),
            reactors,
            reactor_index,
            metric_index: 0,
            selected_metrics: MetricKey::DEFAULTS.into_iter().collect(),
            chart_scale_mode: ChartScaleMode::Normalised,
            focus: FocusPane::Metrics,
            current_data,
            status: "Loaded database".to_owned(),
            connection: Some(connection),
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let reactor_name = self.current_reactor().name.clone();
        let focus = self.focus;
        let connection = open_database(&self.db_path)?;
        validate_schema(&connection)?;
        let reactors = load_reactors(&connection)?;
        let reactor_index = select_reactor(&reactors, Some(&reactor_name))?;
        let current_data = load_reactor_data(&connection, reactors[reactor_index].clone())?;
        let selected_metrics = self.selected_metrics.clone();
        let old_connection = self.connection.replace(connection);
        self.reactors = reactors;
        self.reactor_index = reactor_index;
        self.metric_index = self
            .metric_index
            .min(MetricKey::ALL.len().saturating_sub(1));
        self.focus = focus;
        self.current_data = current_data;
        self.selected_metrics = selected_metrics;
        self.status = "Reloaded database".to_owned();
        drop(old_connection);
        Ok(())
    }

    pub fn set_watch_reload_error(&mut self, error: &str) {
        self.status = format!("Watch reload failed: {error}");
    }

    pub fn current_reactor(&self) -> &ReactorSummary {
        &self.reactors[self.reactor_index]
    }

    pub fn shows_reactor_list(&self) -> bool {
        self.reactors.len() > 1
    }

    pub fn selected_series(&self) -> HashMap<MetricKey, MetricSeries> {
        self.selected_metrics
            .iter()
            .filter_map(|metric| {
                build_series(&self.current_data, *metric).map(|series| (*metric, series))
            })
            .collect()
    }

    pub fn available_metric(&self, metric: MetricKey) -> bool {
        self.current_data.available_metrics.contains(&metric)
    }

    pub fn next_metric(&mut self) {
        self.metric_index = (self.metric_index + 1) % MetricKey::ALL.len();
    }

    pub fn previous_metric(&mut self) {
        self.metric_index = if self.metric_index == 0 {
            MetricKey::ALL.len() - 1
        } else {
            self.metric_index - 1
        };
    }

    pub fn next_reactor(&mut self) -> Result<()> {
        if self.reactors.is_empty() {
            return Ok(());
        }
        self.reactor_index = (self.reactor_index + 1) % self.reactors.len();
        self.refresh_current_data()
    }

    pub fn previous_reactor(&mut self) -> Result<()> {
        if self.reactors.is_empty() {
            return Ok(());
        }
        self.reactor_index = if self.reactor_index == 0 {
            self.reactors.len() - 1
        } else {
            self.reactor_index - 1
        };
        self.refresh_current_data()
    }

    pub fn toggle_metric(&mut self) {
        let metric = MetricKey::ALL[self.metric_index];
        if self.selected_metrics.contains(&metric) {
            self.selected_metrics.remove(&metric);
            self.status = format!("Hid {}", metric.title());
        } else {
            self.selected_metrics.insert(metric);
            self.status = format!("Showing {}", metric.title());
        }
    }

    pub fn hide_all_metrics(&mut self) {
        self.selected_metrics.clear();
        self.status = "Hid all metrics".to_owned();
    }

    pub fn show_all_metrics(&mut self) {
        self.selected_metrics = MetricKey::ALL.into_iter().collect();
        self.status = "Showing all metrics".to_owned();
    }

    pub fn toggle_focus(&mut self) {
        if !self.shows_reactor_list() {
            self.focus = FocusPane::Metrics;
            return;
        }

        self.focus = match self.focus {
            FocusPane::Reactors => FocusPane::Metrics,
            FocusPane::Metrics => FocusPane::Reactors,
        };
    }

    pub fn toggle_chart_scale_mode(&mut self) {
        self.chart_scale_mode = self.chart_scale_mode.toggle();
        self.status = format!("Showing {} chart scale", self.chart_scale_mode.label());
    }

    pub fn latest_raw_value(&self, metric: MetricKey) -> Option<f64> {
        self.current_data
            .points
            .iter()
            .rev()
            .find_map(|point| point.raw_values.get(&metric).copied().flatten())
    }

    pub fn chart_bounds(&self) -> Option<(f64, f64)> {
        let start = self.current_data.points.first()?.ingame_time;
        let end = self.current_data.points.last()?.ingame_time;
        Some((0.0, (end - start).num_seconds().max(1) as f64))
    }

    fn refresh_current_data(&mut self) -> Result<()> {
        let connection = self.connection()?;
        self.current_data =
            load_reactor_data(connection, self.reactors[self.reactor_index].clone())?;
        self.status = format!("Selected reactor {}", self.current_reactor().name);
        Ok(())
    }

    pub fn close_database(&mut self) -> Result<()> {
        if let Some(connection) = self.connection.take() {
            connection.close().map_err(|(_, error)| {
                anyhow!("failed to close SQLite database cleanly: {error}")
            })?;
            self.status = "Closed database".to_owned();
        }
        Ok(())
    }

    fn connection(&self) -> Result<&Connection> {
        self.connection
            .as_ref()
            .ok_or_else(|| anyhow!("database connection is not available"))
    }
}

pub enum AppAction {
    Continue,
    Quit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupSelection {
    WaitForReactor,
    Ready,
}

pub fn handle_key(app: &mut AppState, key: crossterm::event::KeyEvent) -> Result<AppAction> {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Char('q') => Ok(AppAction::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Ok(AppAction::Quit),
        KeyCode::Tab => {
            app.toggle_focus();
            Ok(AppAction::Continue)
        }
        KeyCode::Up => {
            match app.focus {
                FocusPane::Reactors => app.previous_reactor()?,
                FocusPane::Metrics => app.previous_metric(),
            }
            Ok(AppAction::Continue)
        }
        KeyCode::Down => {
            match app.focus {
                FocusPane::Reactors => app.next_reactor()?,
                FocusPane::Metrics => app.next_metric(),
            }
            Ok(AppAction::Continue)
        }
        KeyCode::Left => {
            if app.focus == FocusPane::Metrics {
                app.hide_all_metrics();
            }
            Ok(AppAction::Continue)
        }
        KeyCode::Right => {
            if app.focus == FocusPane::Metrics {
                app.show_all_metrics();
            }
            Ok(AppAction::Continue)
        }
        KeyCode::Char(' ') => {
            if app.focus == FocusPane::Metrics {
                app.toggle_metric();
            }
            Ok(AppAction::Continue)
        }
        KeyCode::Char('r') => {
            app.reload()?;
            Ok(AppAction::Continue)
        }
        KeyCode::Char('n') => {
            app.toggle_chart_scale_mode();
            Ok(AppAction::Continue)
        }
        _ => Ok(AppAction::Continue),
    }
}

pub fn ensure_reactor_selection(
    path: &Path,
    requested_reactor: Option<&str>,
) -> Result<StartupSelection> {
    let connection = open_database(path)?;
    validate_schema(&connection)?;
    let reactors = load_reactors(&connection)?;
    determine_startup_selection(&reactors, requested_reactor)
}

fn determine_startup_selection(
    reactors: &[ReactorSummary],
    requested_reactor: Option<&str>,
) -> Result<StartupSelection> {
    if reactors.is_empty() {
        return Ok(StartupSelection::WaitForReactor);
    }

    if requested_reactor.is_some() {
        select_reactor(&reactors, requested_reactor)?;
    }

    Ok(StartupSelection::Ready)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataPoint, ReactorData};
    use chrono::NaiveDateTime;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn test_app() -> AppState {
        AppState {
            db_path: PathBuf::from("test.db"),
            reactors: vec![ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            }],
            reactor_index: 0,
            metric_index: 0,
            selected_metrics: BTreeSet::from([MetricKey::Temperature]),
            chart_scale_mode: ChartScaleMode::Normalised,
            focus: FocusPane::Metrics,
            current_data: ReactorData {
                reactor: ReactorSummary {
                    id: 1,
                    name: "reactor_a".to_owned(),
                },
                points: vec![DataPoint {
                    ingame_time: NaiveDateTime::parse_from_str(
                        "2026-05-10T22:45:21",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .expect("timestamp"),
                    raw_values: HashMap::from([(MetricKey::Temperature, Some(1.0))]),
                }],
                available_metrics: BTreeSet::from([MetricKey::Temperature]),
            },
            status: String::new(),
            connection: None,
        }
    }

    #[test]
    fn toggling_metric_updates_selection() {
        let mut app = test_app();

        app.toggle_metric();
        assert!(!app.selected_metrics.contains(&MetricKey::Temperature));

        app.toggle_metric();
        assert!(app.selected_metrics.contains(&MetricKey::Temperature));
    }

    #[test]
    fn ctrl_c_requests_quit() {
        let mut app = test_app();

        let action = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        )
        .expect("handle key");

        assert!(matches!(action, AppAction::Quit));
    }

    #[test]
    fn left_arrow_hides_all_metrics() {
        let mut app = test_app();
        app.selected_metrics = MetricKey::ALL.into_iter().collect();

        handle_key(&mut app, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)).expect("handle key");

        assert!(app.selected_metrics.is_empty());
        assert_eq!(app.status, "Hid all metrics");
    }

    #[test]
    fn right_arrow_shows_all_metrics() {
        let mut app = test_app();
        app.selected_metrics = BTreeSet::new();

        handle_key(&mut app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
            .expect("handle key");

        assert_eq!(app.selected_metrics, MetricKey::ALL.into_iter().collect());
        assert_eq!(app.status, "Showing all metrics");
    }

    #[test]
    fn watch_reload_error_updates_status() {
        let mut app = test_app();

        app.set_watch_reload_error("database is locked");

        assert_eq!(app.status, "Watch reload failed: database is locked");
    }

    #[test]
    fn single_reactor_does_not_show_reactor_list() {
        let mut app = test_app();
        app.selected_metrics = BTreeSet::new();

        assert!(!app.shows_reactor_list());
    }

    #[test]
    fn tab_keeps_metrics_focus_when_only_one_reactor_exists() {
        let mut app = test_app();
        app.selected_metrics = BTreeSet::new();

        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)).expect("handle key");

        assert_eq!(app.focus, FocusPane::Metrics);
    }

    #[test]
    fn default_chart_scale_mode_is_normalised() {
        let app = test_app();
        assert_eq!(app.chart_scale_mode, ChartScaleMode::Normalised);
    }

    #[test]
    fn n_toggles_chart_scale_mode() {
        let mut app = test_app();

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
        )
        .expect("handle key");
        assert_eq!(app.chart_scale_mode, ChartScaleMode::Raw);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
        )
        .expect("handle key");
        assert_eq!(app.chart_scale_mode, ChartScaleMode::Normalised);
    }

    #[test]
    fn toggling_chart_scale_mode_updates_status() {
        let mut app = test_app();

        app.toggle_chart_scale_mode();

        assert_eq!(app.status, "Showing raw chart scale");
    }

    #[test]
    fn startup_selection_waits_for_first_reactor() {
        let selection = determine_startup_selection(&[], None).expect("selection should succeed");

        assert_eq!(selection, StartupSelection::WaitForReactor);
    }

    #[test]
    fn startup_selection_accepts_requested_reactor_when_present() {
        let reactors = vec![
            ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            },
            ReactorSummary {
                id: 2,
                name: "reactor_b".to_owned(),
            },
        ];

        let selection = determine_startup_selection(&reactors, Some("reactor_b"))
            .expect("selection should succeed");

        assert_eq!(selection, StartupSelection::Ready);
    }

    #[test]
    fn startup_selection_rejects_unknown_requested_reactor_when_others_exist() {
        let reactors = vec![ReactorSummary {
            id: 1,
            name: "reactor_a".to_owned(),
        }];

        let error = determine_startup_selection(&reactors, Some("reactor_b"))
            .expect_err("selection should fail");

        assert_eq!(
            error.to_string(),
            "reactor `reactor_b` was not found; known reactors: reactor_a"
        );
    }
}
