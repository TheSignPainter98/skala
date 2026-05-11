use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph,
};

use crate::app::{AppState, FocusPane};
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

pub fn render(frame: &mut Frame<'_>, app: &AppState, watch_enabled: bool) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, layout[0], app);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Length(38),
            Constraint::Min(30),
        ])
        .split(layout[1]);

    render_reactors(frame, content[0], app);
    render_metrics(frame, content[1], app);
    render_chart(frame, content[2], app);
    render_footer(frame, layout[2], app, watch_enabled);
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
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Normalised Time Series");

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
                .title("normalised value")
                .bounds([0.0, 1.0])
                .labels(vec![
                    Line::from("0.0"),
                    Line::from("0.5"),
                    Line::from("1.0"),
                ]),
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
                .map(|(timestamp, value)| ((*timestamp - origin).num_seconds() as f64, *value))
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

fn format_elapsed_time_label(seconds: f64) -> String {
    let total_seconds = seconds.round().max(0.0) as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let remaining_seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{remaining_seconds:02}")
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &AppState, watch_enabled: bool) {
    let highlighted = MetricKey::ALL[app.metric_index];
    let latest_value = app
        .latest_raw_value(highlighted)
        .map(|value| format!("{value:.3} {}", highlighted.unit()))
        .unwrap_or_else(|| "no data".to_owned());
    let watch_label = if watch_enabled {
        " | auto-reload 0.25s"
    } else {
        ""
    };
    let footer = Paragraph::new(format!(
        "Tab switch focus | Up/Down move | Left hide all | Right show all | Space toggle metric | r reload | q quit{}\nHighlighted: {} = {} | {}",
        watch_label,
        highlighted.title(),
        latest_value,
        app.status
    ))
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::format_elapsed_time_label;

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
}
