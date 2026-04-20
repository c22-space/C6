// Scope 3 — Other Indirect GHG Emissions
// Standard: GRI 305-3, ISO 14064-1 §5.3.3
// Reference: GHG Protocol Corporate Value Chain (Scope 3) Standard
//
// All 15 categories must be assessed. If excluded, a documented reason is required.
// Categories 1–8: Upstream. Categories 9–15: Downstream.

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// GHG Protocol Scope 3 category definitions
pub const SCOPE3_CATEGORIES: [(u8, &str, &str); 15] = [
    (1,  "Purchased goods and services",          "upstream"),
    (2,  "Capital goods",                          "upstream"),
    (3,  "Fuel and energy-related activities",     "upstream"),
    (4,  "Upstream transportation and distribution","upstream"),
    (5,  "Waste generated in operations",          "upstream"),
    (6,  "Business travel",                        "upstream"),
    (7,  "Employee commuting",                     "upstream"),
    (8,  "Upstream leased assets",                 "upstream"),
    (9,  "Downstream transportation and distribution","downstream"),
    (10, "Processing of sold products",            "downstream"),
    (11, "Use of sold products",                   "downstream"),
    (12, "End-of-life treatment of sold products", "downstream"),
    (13, "Downstream leased assets",               "downstream"),
    (14, "Franchises",                             "downstream"),
    (15, "Investments",                            "downstream"),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope3CategoryTotal {
    pub category: u8,
    pub category_name: String,
    pub direction: String,
    pub total_tco2e: f64,
    pub source_count: u32,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
    pub uncertainty_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope3Total {
    pub gross_tco2e: f64,
    pub upstream_tco2e: f64,
    pub downstream_tco2e: f64,
    pub categories: Vec<Scope3CategoryTotal>,
    pub excluded_categories: Vec<u8>,
    pub combined_uncertainty_pct: f64,
}

/// Aggregate all Scope 3 sources for a period by category.
/// GRI 305-3 requires: gross total, upstream/downstream split, category breakdown.
pub fn aggregate_period(conn: &Connection, period_id: i64) -> Result<Scope3Total> {
    let mut categories = Vec::new();
    let mut gross_tco2e = 0.0f64;
    let mut upstream_tco2e = 0.0f64;
    let mut downstream_tco2e = 0.0f64;
    let mut excluded_categories = Vec::new();
    let mut uncertainty_num_sq = 0.0f64;

    for (cat_num, cat_name, direction) in &SCOPE3_CATEGORIES {
        let row: Option<(f64, i64, Option<String>, f64)> = {
            let mut stmt = conn.prepare(
                "SELECT
                   COALESCE(SUM(CASE WHEN is_excluded = 0 THEN COALESCE(emissions_tco2e, 0) ELSE 0 END), 0),
                   COUNT(*),
                   MIN(CASE WHEN is_excluded = 1 THEN exclusion_reason ELSE NULL END),
                   COALESCE(
                     SQRT(SUM(CASE WHEN is_excluded = 0 THEN
                       POWER(COALESCE(uncertainty_pct, 0) / 100.0 * COALESCE(emissions_tco2e, 0), 2)
                     ELSE 0 END)),
                   0)
                 FROM emission_sources
                 WHERE period_id = ?1 AND scope = 3 AND scope3_category = ?2"
            )?;
            stmt.query_row(params![period_id, *cat_num as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            }).ok()
        };

        let (total, count, excl_reason, cat_uncertainty_sum) = row.unwrap_or((0.0, 0, None, 0.0));

        // A category is "excluded" if it has no sources OR all sources are excluded
        let is_excluded = count == 0 || excl_reason.is_some();

        if is_excluded { excluded_categories.push(*cat_num); }

        if total > 0.0 {
            gross_tco2e += total;
            if *direction == "upstream" { upstream_tco2e += total; }
            else { downstream_tco2e += total; }
            uncertainty_num_sq += cat_uncertainty_sum.powi(2);
        }

        let cat_uncertainty_pct = if total > 0.0 {
            (cat_uncertainty_sum / total) * 100.0
        } else { 0.0 };

        categories.push(Scope3CategoryTotal {
            category: *cat_num,
            category_name: cat_name.to_string(),
            direction: direction.to_string(),
            total_tco2e: total,
            source_count: count as u32,
            is_excluded,
            exclusion_reason: excl_reason,
            uncertainty_pct: cat_uncertainty_pct,
        });
    }

    let combined_uncertainty_pct = if gross_tco2e > 0.0 {
        (uncertainty_num_sq.sqrt() / gross_tco2e) * 100.0
    } else { 0.0 };

    Ok(Scope3Total {
        gross_tco2e,
        upstream_tco2e,
        downstream_tco2e,
        categories,
        excluded_categories,
        combined_uncertainty_pct,
    })
}

/// Calculate a single Scope 3 source.
pub fn calculate_source(conn: &Connection, source_id: i64) -> Result<f64> {
    let (activity_value, ef_value, gwp_value, is_excluded): (f64, f64, f64, i64) = conn.query_row(
        "SELECT activity_value, emission_factor_value, gwp_value, is_excluded
         FROM emission_sources WHERE id = ?1 AND scope = 3",
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
