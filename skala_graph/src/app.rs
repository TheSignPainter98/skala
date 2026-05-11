use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::data::{
    MetricKey, MetricSeries, ReactorData, ReactorSummary, build_series, load_reactor_data,
    load_reactors, open_database, select_reactor, validate_schema,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusPane {
    Reactors,
    Metrics,
}

#[derive(Debug)]
pub struct AppState {
    pub db_path: PathBuf,
    pub reactors: Vec<ReactorSummary>,
    pub reactor_index: usize,
    pub metric_index: usize,
    pub selected_metrics: BTreeSet<MetricKey>,
    pub focus: FocusPane,
    pub current_data: ReactorData,
    pub status: String,
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
            focus: FocusPane::Metrics,
            current_data,
            status: "Loaded database".to_owned(),
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let reactor_name = self.current_reactor().name.clone();
        let focus = self.focus;
        let reloaded = Self::load(&self.db_path, Some(&reactor_name))?;
        let selected_metrics = self.selected_metrics.clone();
        self.reactors = reloaded.reactors;
        self.reactor_index = reloaded.reactor_index;
        self.metric_index = self
            .metric_index
            .min(MetricKey::ALL.len().saturating_sub(1));
        self.focus = focus;
        self.current_data = reloaded.current_data;
        self.selected_metrics = selected_metrics;
        self.status = "Reloaded database".to_owned();
        Ok(())
    }

    pub fn current_reactor(&self) -> &ReactorSummary {
        &self.reactors[self.reactor_index]
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

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Reactors => FocusPane::Metrics,
            FocusPane::Metrics => FocusPane::Reactors,
        };
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
        let connection = open_database(&self.db_path)?;
        self.current_data =
            load_reactor_data(&connection, self.reactors[self.reactor_index].clone())?;
        self.status = format!("Selected reactor {}", self.current_reactor().name);
        Ok(())
    }
}

pub enum AppAction {
    Continue,
    Quit,
}

pub fn handle_key(app: &mut AppState, key: crossterm::event::KeyEvent) -> Result<AppAction> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('q') => Ok(AppAction::Quit),
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
        _ => Ok(AppAction::Continue),
    }
}

pub fn ensure_reactor_selection(path: &Path, requested_reactor: Option<&str>) -> Result<()> {
    let connection = open_database(path)?;
    validate_schema(&connection)?;
    let reactors = load_reactors(&connection)?;

    if reactors.is_empty() {
        return Err(anyhow!(
            "the database does not contain any reactors with events"
        ));
    }

    if requested_reactor.is_some() {
        select_reactor(&reactors, requested_reactor)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataPoint, ReactorData};
    use chrono::NaiveDateTime;

    #[test]
    fn toggling_metric_updates_selection() {
        let mut app = AppState {
            db_path: PathBuf::from("test.db"),
            reactors: vec![ReactorSummary {
                id: 1,
                name: "reactor_a".to_owned(),
            }],
            reactor_index: 0,
            metric_index: 0,
            selected_metrics: BTreeSet::from([MetricKey::Temperature]),
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
        };

        app.toggle_metric();
        assert!(!app.selected_metrics.contains(&MetricKey::Temperature));

        app.toggle_metric();
        assert!(app.selected_metrics.contains(&MetricKey::Temperature));
    }
}
