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
    let first = app.current_data.points.first().expect("non-empty points");
    let last = app.current_data.points.last().expect("non-empty points");

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(
            Axis::default()
                .title("ingame_time")
                .bounds([x_bounds.0, x_bounds.1])
                .labels(vec![
                    Line::from(first.ingame_time.format("%H:%M:%S").to_string()),
                    Line::from(last.ingame_time.format("%H:%M:%S").to_string()),
                ]),
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let highlighted = MetricKey::ALL[app.metric_index];
    let latest_value = app
        .latest_raw_value(highlighted)
        .map(|value| format!("{value:.3} {}", highlighted.unit()))
        .unwrap_or_else(|| "no data".to_owned());
    let footer = Paragraph::new(format!(
        "Tab switch focus | Up/Down move | Left hide all | Right show all | Space toggle metric | r reload | q quit\nHighlighted: {} = {} | {}",
        highlighted.title(),
        latest_value,
        app.status
    ))
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(footer, area);
}
