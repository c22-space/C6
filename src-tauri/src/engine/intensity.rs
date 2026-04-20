// GRI 305-4: GHG Emissions Intensity
// Intensity ratio = Total emissions / Activity metric (e.g. revenue, production units)
// GRI 305-4 requires: which scopes included, denominator metric, Scope 3 intensity separate

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntensityResult {
    pub includes_scope1: bool,
    pub includes_scope2: bool,
    pub includes_scope3: bool,
    pub total_emissions_tco2e: f64,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: String,
    /// tCO₂e per unit of metric
    pub intensity_ratio: f64,
    /// GRI 305-4: Scope 3 intensity must be reported separately if calculated
    pub scope3_intensity_ratio: Option<f64>,
}

/// Calculate intensity ratio and store it.
pub fn calculate_and_store(
    conn: &Connection,
    period_id: i64,
    includes_scope1: bool,
    includes_scope2: bool,
    includes_scope3: bool,
    metric_name: &str,
    metric_value: f64,
    metric_unit: &str,
    scope1_tco2e: f64,
    scope2_location_tco2e: f64,
    scope3_tco2e: f64,
) -> Result<IntensityResult> {
    let mut total = 0.0f64;
    if includes_scope1 { total += scope1_tco2e; }
    if includes_scope2 { total += scope2_location_tco2e; }
    if includes_scope3 { total += scope3_tco2e; }

    let intensity_ratio = if metric_value > 0.0 { total / metric_value } else { 0.0 };
    let scope3_intensity_ratio = if includes_scope3 && metric_value > 0.0 {
        Some(scope3_tco2e / metric_value)
    } else {
        None
    };

    // Upsert into intensity_ratios table
    conn.execute(
        "INSERT OR REPLACE INTO intensity_ratios
         (period_id, includes_scope1, includes_scope2, includes_scope3,
          total_emissions_tco2e, metric_name, metric_value, metric_unit,
          intensity_ratio, scope3_intensity_ratio)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            period_id,
            includes_scope1 as i64,
            includes_scope2 as i64,
            includes_scope3 as i64,
            total,
            metric_name,
            metric_value,
            metric_unit,
            intensity_ratio,
            scope3_intensity_ratio,
        ],
    )?;

    Ok(IntensityResult {
        includes_scope1,
        includes_scope2,
        includes_scope3,
        total_emissions_tco2e: total,
        metric_name: metric_name.to_string(),
        metric_value,
        metric_unit: metric_unit.to_string(),
        intensity_ratio,
        scope3_intensity_ratio,
    })
}
