use std::path::PathBuf;

use skala_graph::app::AppState;
use skala_graph::data::MetricKey;

fn sample_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("qwen-2.5-positive-meltdown.db")
}

#[test]
fn sample_database_loads_with_expected_defaults() {
    let db_path = sample_db_path();
    assert!(
        db_path.exists(),
        "sample database should exist at {}",
        db_path.display()
    );

    let app = AppState::load(&db_path, Some("reactor_53")).expect("load sample database");

    assert_eq!(app.reactors.len(), 1);
    assert_eq!(app.current_reactor().name, "reactor_53");
    assert!(!app.current_data.points.is_empty());
    assert!(app.selected_metrics.contains(&MetricKey::Temperature));
    assert!(app.selected_metrics.contains(&MetricKey::ActualBurnRate));
    assert!(app.selected_metrics.contains(&MetricKey::TargetBurnRate));
    assert!(
        app.selected_metrics
            .contains(&MetricKey::EnergyProductionRate)
    );
}
