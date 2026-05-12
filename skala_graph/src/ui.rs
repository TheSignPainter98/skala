use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph,
};

use crate::app::{AppState, ChartScaleMode, FocusPane};
use crate::data::{MetricKey, MetricSeries};

const PALETTE: [Color; 8] = [
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Blue,
    Color::Red,
    Color::LightCyan,
    Color::LightGreen,
];

pub fn render(frame: &mut Frame<'_>, app: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, layout[0], app);

    let content = if app.shows_reactor_list() {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(26),
                Constraint::Length(38),
                Constraint::Min(30),
            ])
            .split(layout[1])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(38), Constraint::Min(30)])
            .split(layout[1])
    };

    if app.shows_reactor_list() {
        render_reactors(frame, content[0], app);
        render_metrics(frame, content[1], app);
        render_chart(frame, content[2], app);
    } else {
        render_metrics(frame, content[0], app);
        render_chart(frame, content[1], app);
    }

    render_footer(frame, layout[2], app);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let title = Paragraph::new(format!(
        "Database: {} | Reactor: {}",
        app.db_path.display(),
        app.current_reactor().name
    ))
    .block(Block::default().borders(Borders::ALL).title("SKALA Graph"))
    .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(title, area);
}

fn render_reactors(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let items = app
        .reactors
        .iter()
        .map(|reactor| ListItem::new(reactor.name.clone()))
        .collect::<Vec<_>>();
    let mut state = ListState::default().with_selected(Some(app.reactor_index));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(if app.focus == FocusPane::Reactors {
            "Reactors [focus]"
        } else {
            "Reactors"
        });
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_metrics(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let items = MetricKey::ALL
        .iter()
        .enumerate()
        .map(|(index, metric)| {
            let checked = if app.selected_metrics.contains(metric) {
                "[x]"
            } else {
                "[ ]"
            };
            let availability = if app.available_metric(*metric) {
                ""
            } else {
                " (no data)"
            };
            let line = format!("{checked} {}{availability}", metric.title());
            let style = if index == app.metric_index && app.focus == FocusPane::Metrics {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if !app.available_metric(*metric) {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(Line::styled(line, style))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default().with_selected(Some(app.metric_index));
    let list =
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(
                if app.focus == FocusPane::Metrics {
                    "Metrics [focus]"
                } else {
                    "Metrics"
                },
            ))
            .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_chart(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let series = app.selected_series();
    let y_axis = chart_y_axis_bounds(app.chart_scale_mode, &series);
    let chart_title = match app.chart_scale_mode {
        ChartScaleMode::Normalised => "Selected Metrics (Normalised)",
        ChartScaleMode::Raw => "Selected Metrics (Raw)",
    };
    let y_axis_title = match app.chart_scale_mode {
        ChartScaleMode::Normalised => "normalised value",
        ChartScaleMode::Raw => "raw value (shared axis)",
    };
    let block = Block::default().borders(Borders::ALL).title(chart_title);

    if app.current_data.points.is_empty() {
        frame.render_widget(
            Paragraph::new("This reactor has no event data.")
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    if app.selected_metrics.is_empty() {
        frame.render_widget(
            Paragraph::new("Select one or more metrics with Space.")
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let datasets = chart_datasets(app, &series);
    if datasets.is_empty() {
        frame.render_widget(
            Paragraph::new("Selected metrics do not have values for this reactor.")
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let x_bounds = app.chart_bounds().unwrap_or((0.0, 1.0));
    let end_label = format_elapsed_time_label(x_bounds.1);

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(
            Axis::default()
                .title("elapsed ingame time")
                .bounds([x_bounds.0, x_bounds.1])
                .labels(vec![Line::from("00:00:00"), Line::from(end_label)]),
        )
        .y_axis(
            Axis::default()
                .title(y_axis_title)
                .bounds([y_axis.0, y_axis.1])
                .labels(chart_y_axis_labels(y_axis)),
        );

    frame.render_widget(chart, area);
}

fn chart_datasets<'a>(
    app: &AppState,
    series: &'a HashMap<MetricKey, MetricSeries>,
) -> Vec<Dataset<'a>> {
    let origin = match app.current_data.points.first() {
        Some(point) => point.ingame_time,
        None => return Vec::new(),
    };

    app.selected_metrics
        .iter()
        .enumerate()
        .filter_map(|(index, metric)| {
            let series = series.get(metric)?;
            let points = series
                .points
                .iter()
                .map(|(timestamp, value)| {
                    (
                        (*timestamp - origin).num_seconds() as f64,
                        plotted_value(app.chart_scale_mode, series, *value),
                    )
                })
                .collect::<Vec<_>>();
            Some(
                Dataset::default()
                    .name(metric.title())
                    .graph_type(GraphType::Line)
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(PALETTE[index % PALETTE.len()]))
                    .data(Box::leak(points.into_boxed_slice())),
            )
        })
        .collect()
}

fn plotted_value(mode: ChartScaleMode, series: &MetricSeries, raw_value: f64) -> f64 {
    match mode {
        ChartScaleMode::Normalised => {
            if series.is_constant {
                0.5
            } else {
                (raw_value - series.raw_min) / (series.raw_max - series.raw_min)
            }
        }
        ChartScaleMode::Raw => raw_value,
    }
}

fn chart_y_axis_bounds(
    mode: ChartScaleMode,
    series: &HashMap<MetricKey, MetricSeries>,
) -> (f64, f64) {
    match mode {
        ChartScaleMode::Normalised => (0.0, 1.0),
        ChartScaleMode::Raw => {
            let Some(maximum) = series
                .values()
                .map(|metric| metric.raw_max)
                .reduce(f64::max)
            else {
                return (0.0, 1.0);
            };

            if maximum <= 0.0 {
                (0.0, 1.0)
            } else {
                (0.0, maximum)
            }
        }
    }
}

fn chart_y_axis_labels(bounds: (f64, f64)) -> Vec<Line<'static>> {
    let midpoint = bounds.0 + ((bounds.1 - bounds.0) / 2.0);
    vec![
        Line::from(format!("{:.3}", bounds.0)),
        Line::from(format!("{midpoint:.3}")),
        Line::from(format!("{:.3}", bounds.1)),
    ]
}

fn format_elapsed_time_label(seconds: f64) -> String {
    let total_seconds = seconds.round().max(0.0) as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let remaining_seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{remaining_seconds:02}")
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let highlighted = MetricKey::ALL[app.metric_index];
    let latest_value = app
        .latest_raw_value(highlighted)
        .map(|value| format!("{value:.3} {}", highlighted.unit()))
        .unwrap_or_else(|| "no data".to_owned());
    let controls = if app.shows_reactor_list() {
        "Tab switch focus | Up/Down move"
    } else {
        "Up/Down move"
    };
    let footer = Paragraph::new(format!(
        "{} | Left hide all | Right show all | Space toggle metric | n toggle scale | r reload | auto-reload 0.25s | q quit\nPlot mode: {} | Highlighted: {} = {} | {}",
        controls,
        app.chart_scale_mode.label(),
        highlighted.title(),
        latest_value,
        app.status
    ))
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{chart_y_axis_bounds, format_elapsed_time_label, plotted_value};
    use crate::app::ChartScaleMode;
    use crate::data::{MetricKey, MetricSeries};
    use chrono::NaiveDateTime;

    fn metric_series(
        key: MetricKey,
        points: &[(&str, f64)],
        raw_min: f64,
        raw_max: f64,
        is_constant: bool,
    ) -> MetricSeries {
        MetricSeries {
            key,
            points: points
                .iter()
                .map(|(timestamp, value)| {
                    (
                        NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
                            .expect("timestamp"),
                        *value,
                    )
                })
                .collect(),
            raw_min,
            raw_max,
            is_constant,
        }
    }

    #[test]
    fn elapsed_time_label_starts_from_zero() {
        assert_eq!(format_elapsed_time_label(0.0), "00:00:00");
    }

    #[test]
    fn elapsed_time_label_rounds_to_whole_seconds() {
        assert_eq!(format_elapsed_time_label(12.6), "00:00:13");
    }

    #[test]
    fn elapsed_time_label_formats_hours_minutes_and_seconds() {
        assert_eq!(format_elapsed_time_label(3661.0), "01:01:01");
    }

    #[test]
    fn normalised_mode_uses_fixed_y_axis_bounds() {
        let bounds = chart_y_axis_bounds(ChartScaleMode::Normalised, &HashMap::new());
        assert_eq!(bounds, (0.0, 1.0));
    }

    #[test]
    fn raw_mode_uses_combined_bounds_across_selected_series() {
        let series = HashMap::from([
            (
                MetricKey::Temperature,
                metric_series(
                    MetricKey::Temperature,
                    &[("2026-05-10T22:45:21", 5.0), ("2026-05-10T22:45:31", 15.0)],
                    5.0,
                    15.0,
                    false,
                ),
            ),
            (
                MetricKey::ActualBurnRate,
                metric_series(
                    MetricKey::ActualBurnRate,
                    &[("2026-05-10T22:45:21", 1.0), ("2026-05-10T22:45:31", 2.0)],
                    1.0,
                    2.0,
                    false,
                ),
            ),
        ]);

        let bounds = chart_y_axis_bounds(ChartScaleMode::Raw, &series);

        assert_eq!(bounds, (0.0, 15.0));
    }

    #[test]
    fn constant_zero_raw_series_get_padded_bounds() {
        let series = HashMap::from([(
            MetricKey::TargetBurnRate,
            metric_series(
                MetricKey::TargetBurnRate,
                &[("2026-05-10T22:45:21", 0.0), ("2026-05-10T22:45:31", 0.0)],
                0.0,
                0.0,
                true,
            ),
        )]);

        let bounds = chart_y_axis_bounds(ChartScaleMode::Raw, &series);

        assert_eq!(bounds, (0.0, 1.0));
    }

    #[test]
    fn sparse_series_keep_missing_points_out_of_raw_bounds() {
        let sparse = metric_series(
            MetricKey::AdviceNewTargetBurnRate,
            &[("2026-05-10T22:45:31", 30.0)],
            30.0,
            30.0,
            true,
        );

        let bounds = chart_y_axis_bounds(
            ChartScaleMode::Raw,
            &HashMap::from([(MetricKey::AdviceNewTargetBurnRate, sparse.clone())]),
        );

        assert_eq!(bounds, (0.0, 30.0));
        assert_eq!(
            plotted_value(ChartScaleMode::Raw, &sparse, sparse.points[0].1),
            30.0
        );
        assert_eq!(
            plotted_value(ChartScaleMode::Normalised, &sparse, sparse.points[0].1),
            0.5
        );
    }

    #[test]
    fn raw_mode_uses_highest_visible_value_even_with_lower_non_zero_series() {
        let series = HashMap::from([
            (
                MetricKey::Temperature,
                metric_series(
                    MetricKey::Temperature,
                    &[("2026-05-10T22:45:21", 20.0), ("2026-05-10T22:45:31", 25.0)],
                    20.0,
                    25.0,
                    false,
                ),
            ),
            (
                MetricKey::ActualBurnRate,
                metric_series(
                    MetricKey::ActualBurnRate,
                    &[("2026-05-10T22:45:21", 2.0), ("2026-05-10T22:45:31", 3.0)],
                    2.0,
                    3.0,
                    false,
                ),
            ),
        ]);

        let bounds = chart_y_axis_bounds(ChartScaleMode::Raw, &series);

        assert_eq!(bounds, (0.0, 25.0));
    }
}
