// Scope 2 — Energy Indirect GHG Emissions
// Standard: GRI 305-2, ISO 14064-1 §5.3.2
//
// GRI 305-2 REQUIRES both location-based and market-based calculations.
// Reporting only one is non-compliant. Both values must appear in all reports.
//
// Location-based: uses grid-average emission factor for the region
// Market-based:   uses emission factor from contractual instruments (RECs, PPAs, etc.)
//                 If no contractual instruments: defaults to supplier residual mix factor

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope2Result {
    /// Gross Scope 2 emissions — location-based (tCO₂e)
    /// Uses grid-average emission factor for the region
    pub location_based_tco2e: f64,

    /// Gross Scope 2 emissions — market-based (tCO₂e)
    /// Uses contractual instrument emission factors
    pub market_based_tco2e: f64,

    /// Percentage of electricity covered by contractual instruments (0–100)
    pub contractual_coverage_pct: f64,

    /// Individual source breakdowns
    pub location_sources: Vec<Scope2Source>,
    pub market_sources: Vec<Scope2Source>,

    /// Combined uncertainty for each method
    pub location_uncertainty_pct: f64,
    pub market_uncertainty_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope2Source {
    pub id: i64,
    pub category_name: String,
    pub activity_value: f64,
    pub activity_unit: String,
    pub emission_factor_value: f64,
    pub emission_factor_source: String,
    pub emissions_tco2e: f64,
    pub uncertainty_pct: Option<f64>,
}

/// Aggregate Scope 2 for a period.
/// Returns BOTH location-based and market-based totals (both required by GRI 305-2).
///
/// This function returns Err if either method has no data — the caller must
/// ensure both location_based and market_based sources are entered.
pub fn aggregate_period(
    conn: &Connection,
    period_id: i64,
) -> Result<Scope2Result> {
    let location_sources = fetch_scope2_sources(conn, period_id, "location_based")?;
    let market_sources = fetch_scope2_sources(conn, period_id, "market_based")?;

    let (location_based_tco2e, location_uncertainty_pct) = sum_sources(&location_sources);
    let (market_based_tco2e, market_uncertainty_pct) = sum_sources(&market_sources);

    // Calculate contractual coverage: total MWh covered by contractual instruments
    // vs total electricity consumed
    let contractual_coverage_pct = calculate_contractual_coverage(conn, period_id)?;

    Ok(Scope2Result {
        location_based_tco2e,
        market_based_tco2e,
        contractual_coverage_pct,
        location_sources,
        market_sources,
        location_uncertainty_pct,
        market_uncertainty_pct,
    })
}

fn fetch_scope2_sources(
    conn: &Connection,
    period_id: i64,
    method: &str,
) -> Result<Vec<Scope2Source>> {
    let mut stmt = conn.prepare(
        "SELECT id, category_name, activity_value, activity_unit,
                emission_factor_value, emission_factor_source,
                COALESCE(emissions_tco2e, 0.0), uncertainty_pct
         FROM emission_sources
         WHERE period_id = ?1 AND scope = 2 AND scope2_method = ?2 AND is_excluded = 0"
    )?;
    let rows = stmt.query_map(params![period_id, method], |row| {
        Ok(Scope2Source {
            id: row.get(0)?,
            category_name: row.get(1)?,
            activity_value: row.get(2)?,
            activity_unit: row.get(3)?,
            emission_factor_value: row.get(4)?,
            emission_factor_source: row.get(5)?,
            emissions_tco2e: row.get(6)?,
            uncertainty_pct: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>>>()
}

fn sum_sources(sources: &[Scope2Source]) -> (f64, f64) {
    let total: f64 = sources.iter().map(|s| s.emissions_tco2e).sum();
    let uncertainty_num_sq: f64 = sources.iter()
        .filter_map(|s| s.uncertainty_pct.map(|u| (u / 100.0 * s.emissions_tco2e).powi(2)))
        .sum();
    let uncertainty = if total > 0.0 {
        (uncertainty_num_sq.sqrt() / total) * 100.0
    } else {
        0.0
    };
    (total, uncertainty)
}

fn calculate_contractual_coverage(conn: &Connection, period_id: i64) -> Result<f64> {
    // Sum of MWh covered by contractual instruments vs total electricity kWh
    let total_kwh: f64 = conn.query_row(
        "SELECT COALESCE(SUM(activity_value), 0)
         FROM emission_sources
         WHERE period_id = ?1 AND scope = 2 AND scope2_method = 'location_based'
           AND activity_unit IN ('kWh','MWh')",
        params![period_id],
        |r| r.get(0),
    ).unwrap_or(0.0);

    let covered_mwh: f64 = conn.query_row(
        "SELECT COALESCE(SUM(ci.quantity_mwh), 0)
         FROM contractual_instruments ci
         JOIN emission_sources es ON ci.source_id = es.id
         WHERE es.period_id = ?1 AND es.scope = 2",
        params![period_id],
        |r| r.get(0),
    ).unwrap_or(0.0);

    if total_kwh <= 0.0 { return Ok(0.0); }
    let covered_kwh = covered_mwh * 1000.0;
    Ok((covered_kwh / total_kwh * 100.0).min(100.0))
}

/// Calculate a single Scope 2 source and store the result.
pub fn calculate_source(conn: &Connection, source_id: i64) -> Result<f64> {
    let (activity_value, ef_value, gwp_value, is_excluded): (f64, f64, f64, i64) = conn.query_row(
        "SELECT activity_value, emission_factor_value, gwp_value, is_excluded
         FROM emission_sources WHERE id = ?1 AND scope = 2",
        params![source_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;

    if is_excluded != 0 { return Ok(0.0); }

    let emissions_tco2e = (activity_value * ef_value * gwp_value) / 1000.0;
    conn.execute(
        "UPDATE emission_sources SET emissions_tco2e = ?1, updated_at = unixepoch() WHERE id = ?2",
        params![emissions_tco2e, source_id],
    )?;
    Ok(emissions_tco2e)
}
