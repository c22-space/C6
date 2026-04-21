pub mod scope1;
pub mod scope2;
pub mod scope3;
pub mod intensity;

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Full GHG inventory result for a reporting period.
/// Aggregates all scopes per GRI 305 and ISO 14064-1.
#[derive(Debug, Serialize, Deserialize)]
pub struct PeriodInventory {
    pub period_id: i64,
    pub year: i64,
    pub gwp_ar_version: String,
    pub scope1: scope1::Scope1Total,
    pub scope2: scope2::Scope2Result,
    pub scope3: scope3::Scope3Total,
    /// Combined total: Scope 1 + Scope 2 (location-based) + Scope 3
    pub total_tco2e: f64,
    /// Scope 1 + 2 only (most commonly reported)
    pub scope1_scope2_tco2e: f64,
}

/// Run the full calculation for a reporting period.
/// Recalculates all sources, then aggregates.
pub fn calculate_period(conn: &Connection, period_id: i64) -> Result<PeriodInventory> {
    let (year, gwp_ar): (i64, String) = conn.query_row(
        "SELECT year, gwp_ar_version FROM reporting_periods WHERE id = ?1",
        params![period_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    // Recalculate all sources for this period
    let source_ids: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT id FROM emission_sources WHERE period_id = ?1 AND is_excluded = 0"
        )?;
        let rows = stmt.query_map(params![period_id], |r| r.get(0))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    for id in &source_ids {
        let scope: i64 = conn.query_row(
            "SELECT scope FROM emission_sources WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        match scope {
            1 => { scope1::calculate_source(conn, *id)?; }
            2 => { scope2::calculate_source(conn, *id)?; }
            3 => { scope3::calculate_source(conn, *id)?; }
            _ => {}
        }
    }

    let scope1 = scope1::aggregate_period(conn, period_id)?;
    let scope2 = scope2::aggregate_period(conn, period_id)?;
    let scope3 = scope3::aggregate_period(conn, period_id)?;

    let scope1_scope2_tco2e = scope1.gross_tco2e + scope2.location_based_tco2e;
    let total_tco2e = scope1_scope2_tco2e + scope3.gross_tco2e;

    Ok(PeriodInventory {
        period_id,
        year,
        gwp_ar_version: gwp_ar,
        scope1,
        scope2,
        scope3,
        total_tco2e,
        scope1_scope2_tco2e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use rusqlite::params;

    fn setup(conn: &rusqlite::Connection) -> i64 {
        conn.execute(
            "INSERT INTO organizations (name, boundary_method) VALUES ('Acme', 'operational_control')",
            [],
        ).unwrap();
        let org_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO entities (org_id, name, type) VALUES (?1, 'HQ', 'facility')",
            params![org_id],
        ).unwrap();
        let entity_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO reporting_periods (org_id, year, start_date, end_date, gwp_ar_version)
             VALUES (?1, 2024, '2024-01-01', '2024-12-31', 'AR6')",
            params![org_id],
        ).unwrap();
        let period_id = conn.last_insert_rowid();

        // Scope 1: 1000 kWh × 0.2 × 1.0 / 1000 = 0.2 tCO2e
        conn.execute(
            "INSERT INTO emission_sources
             (entity_id, period_id, scope, category_name, ghg_type,
              activity_value, activity_unit, emission_factor_value,
              emission_factor_unit, emission_factor_source, gwp_value)
             VALUES (?1, ?2, 1, 'Natural Gas', 'CO2', 1000.0, 'kWh', 0.2, 'kgCO2e/kWh', 'IPCC', 1.0)",
            params![entity_id, period_id],
        ).unwrap();

        // Scope 2 location-based: 500 kWh × 0.4 × 1.0 / 1000 = 0.2 tCO2e
        conn.execute(
            "INSERT INTO emission_sources
             (entity_id, period_id, scope, scope2_method, category_name, ghg_type,
              activity_value, activity_unit, emission_factor_value,
              emission_factor_unit, emission_factor_source, gwp_value)
             VALUES (?1, ?2, 2, 'location_based', 'Grid Electricity', 'CO2',
                     500.0, 'kWh', 0.4, 'kgCO2e/kWh', 'DEFRA', 1.0)",
            params![entity_id, period_id],
        ).unwrap();

        // Scope 2 market-based: 500 kWh × 0.1 × 1.0 / 1000 = 0.05 tCO2e
        conn.execute(
            "INSERT INTO emission_sources
             (entity_id, period_id, scope, scope2_method, category_name, ghg_type,
              activity_value, activity_unit, emission_factor_value,
              emission_factor_unit, emission_factor_source, gwp_value)
             VALUES (?1, ?2, 2, 'market_based', 'Grid Electricity', 'CO2',
                     500.0, 'kWh', 0.1, 'kgCO2e/kWh', 'Supplier', 1.0)",
            params![entity_id, period_id],
        ).unwrap();

        // Scope 3 cat 6 (business travel): 2000 km × 0.15 × 1.0 / 1000 = 0.3 tCO2e
        conn.execute(
            "INSERT INTO emission_sources
             (entity_id, period_id, scope, scope3_category, category_name, ghg_type,
              activity_value, activity_unit, emission_factor_value,
              emission_factor_unit, emission_factor_source, gwp_value)
             VALUES (?1, ?2, 3, 6, 'Business Travel', 'CO2',
                     2000.0, 'km', 0.15, 'kgCO2e/km', 'IPCC', 1.0)",
            params![entity_id, period_id],
        ).unwrap();

        period_id
    }

    #[test]
    fn calculate_period_totals_all_scopes() {
        let conn = test_conn();
        let period_id = setup(&conn);
        let inv = calculate_period(&conn, period_id).unwrap();

        assert!((inv.scope1.gross_tco2e - 0.2).abs() < 1e-9, "scope1={}", inv.scope1.gross_tco2e);
        assert!((inv.scope2.location_based_tco2e - 0.2).abs() < 1e-9);
        assert!((inv.scope3.gross_tco2e - 0.3).abs() < 1e-9);
        // scope1 + scope2 location-based = 0.4
        assert!((inv.scope1_scope2_tco2e - 0.4).abs() < 1e-9);
        // total = 0.2 + 0.2 + 0.3 = 0.7
        assert!((inv.total_tco2e - 0.7).abs() < 1e-9);
    }

    #[test]
    fn calculate_period_year_and_gwp_version_preserved() {
        let conn = test_conn();
        let period_id = setup(&conn);
        let inv = calculate_period(&conn, period_id).unwrap();
        assert_eq!(inv.year, 2024);
        assert_eq!(inv.gwp_ar_version, "AR6");
    }
}
