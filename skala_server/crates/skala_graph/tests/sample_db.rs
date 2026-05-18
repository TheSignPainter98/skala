use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use skala_graph::app::AppState;
use skala_graph::data::MetricKey;

fn sample_db_path() -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("results")
        .join("qwen-2.5-positive-meltdown.db")
}

#[tokio::test(flavor = "current_thread")]
async fn sample_database_loads_with_expected_defaults() {
    let db_path = sample_db_path();
    assert!(
        db_path.exists(),
        "sample database should exist at {db_path}",
    );

    let app = AppState::load(&db_path, Some("reactor_53"))
        .await
        .expect("load sample database");

    assert_eq!(app.reactors.len(), 1);
    assert_eq!(app.current_reactor().name, "reactor_53");
    assert!(!app.current_data.points.is_empty());
    assert_eq!(
        app.selected_metrics,
        BTreeSet::from([
            MetricKey::DamagePercent,
            MetricKey::EnergyProductionRate,
            MetricKey::AdviceNewTargetReactivity,
            MetricKey::ProductionTargetRate,
        ])
    );
}
