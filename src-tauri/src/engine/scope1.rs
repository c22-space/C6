// Scope 1 — Direct GHG Emissions
// Standard: GRI 305-1, ISO 14064-1 §5.3.1
//
// Calculation: Emissions (tCO₂e) = Activity Data × Emission Factor × GWP
// Result excludes biogenic CO₂ (tracked separately per GRI 305-1)

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope1Source {
    pub id: i64,
    pub category_name: String,
    pub ghg_type: String,
    pub activity_value: f64,
    pub activity_unit: String,
    pub emission_factor_value: f64,
    pub emission_factor_unit: String,
    pub emission_factor_source: String,
    pub gwp_value: f64,
    pub emissions_tco2e: Option<f64>,
    pub biogenic_co2_tco2e: Option<f64>,
    pub uncertainty_pct: Option<f64>,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope1Total {
    pub gross_tco2e: f64,
    pub biogenic_co2_tco2e: f64,
    pub by_gas: std::collections::HashMap<String, f64>,
    pub sources: Vec<Scope1Source>,
    pub combined_uncertainty_pct: f64,
}

/// Calculate and store Scope 1 emissions for a source.
/// Returns the calculated tCO₂e value.
///
/// Formula: tCO₂e = (activity_value × emission_factor_value × gwp_value) / 1000
/// (emission_factor in kgCO₂e/unit → divide by 1000 to convert to tCO₂e)
pub fn calculate_source(
    conn: &Connection,
    source_id: i64,
) -> Result<f64> {
    let (activity_value, ef_value, gwp_value, is_excluded): (f64, f64, f64, i64) = conn.query_row(
        "SELECT activity_value, emission_factor_value, gwp_value, is_excluded
         FROM emission_sources WHERE id = ?1 AND scope = 1",
        params![source_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;

    if is_excluded != 0 {
        return Ok(0.0);
    }

    // Core GHG calculation: Activity × Emission Factor × GWP / 1000 (kg → t)
    let emissions_tco2e = (activity_value * ef_value * gwp_value) / 1000.0;

    // Store the calculated result
    conn.execute(
        "UPDATE emission_sources SET emissions_tco2e = ?1, updated_at = unixepoch() WHERE id = ?2",
        params![emissions_tco2e, source_id],
    )?;

    Ok(emissions_tco2e)
}

/// Aggregate all Scope 1 sources for a reporting period.
/// GRI 305-1 requires: gross total, biogenic CO₂ separate, breakdown by gas type.
pub fn aggregate_period(
    conn: &Connection,
    period_id: i64,
) -> Result<Scope1Total> {
    let sources: Vec<Scope1Source> = {
        let mut stmt = conn.prepare(
            "SELECT id, category_name, ghg_type, activity_value, activity_unit,
                    emission_factor_value, emission_factor_unit, emission_factor_source,
                    gwp_value, emissions_tco2e, biogenic_co2_tco2e, uncertainty_pct,
                    is_excluded, exclusion_reason
             FROM emission_sources
             WHERE period_id = ?1 AND scope = 1
             ORDER BY category_name"
        )?;
        let rows = stmt.query_map(params![period_id], |row| {
            Ok(Scope1Source {
                id: row.get(0)?,
                category_name: row.get(1)?,
                ghg_type: row.get(2)?,
                activity_value: row.get(3)?,
                activity_unit: row.get(4)?,
                emission_factor_value: row.get(5)?,
                emission_factor_unit: row.get(6)?,
                emission_factor_source: row.get(7)?,
                gwp_value: row.get(8)?,
                emissions_tco2e: row.get(9)?,
                biogenic_co2_tco2e: row.get(10)?,
                uncertainty_pct: row.get(11)?,
                is_excluded: row.get::<_, i64>(12)? != 0,
                exclusion_reason: row.get(13)?,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()?
    };

    let mut gross_tco2e = 0.0f64;
    let mut biogenic_co2_tco2e = 0.0f64;
    let mut by_gas: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    // For combined uncertainty: weighted quadrature sum (ISO 14064-1 §7.4)
    let mut uncertainty_numerator_sq = 0.0f64;

    for s in &sources {
        if s.is_excluded { continue; }
        let emission = s.emissions_tco2e.unwrap_or(0.0);
        gross_tco2e += emission;
        biogenic_co2_tco2e += s.biogenic_co2_tco2e.unwrap_or(0.0);
        *by_gas.entry(s.ghg_type.clone()).or_insert(0.0) += emission;

        if let Some(u) = s.uncertainty_pct {
            uncertainty_numerator_sq += (u / 100.0 * emission).powi(2);
        }
    }

    // Combined uncertainty = √(Σ(u_i × e_i)²) / total × 100
    let combined_uncertainty_pct = if gross_tco2e > 0.0 {
        (uncertainty_numerator_sq.sqrt() / gross_tco2e) * 100.0
    } else {
        0.0
    };

    Ok(Scope1Total {
        gross_tco2e,
        biogenic_co2_tco2e,
        by_gas,
        sources,
        combined_uncertainty_pct,
    })
}
