#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, serde::Serialize)]
#[serde(tag = "integrity")]
#[serde(rename_all = "kebab-case")]
#[quicktype(namespace = "server")]
pub enum TurbineSnapshot {
    Intact(IntactTurbineSnapshot),
    Destroyed,
}

#[derive(Clone, Debug, quicktype::Quicktype, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub struct IntactTurbineSnapshot {
    pub stored_kinetic_energy: f64,
    pub energy_production_rate: f64,
}
