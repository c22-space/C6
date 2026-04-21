use crate::db::Database;
use crate::engine;
use crate::engine::intensity;
use crate::error::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Run the full GHG inventory calculation for a reporting period.
#[tauri::command]
pub fn calculate_period(
    db: State<Database>,
    period_id: i64,
) -> Result<engine::PeriodInventory> {
    let conn = db.0.lock().unwrap();
    Ok(engine::calculate_period(&conn, period_id)?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntensityInput {
    pub includes_scope1: bool,
    pub includes_scope2: bool,
    pub includes_scope3: bool,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: String,
    pub scope1_tco2e: f64,
    pub scope2_location_tco2e: f64,
    pub scope3_tco2e: f64,
}

/// Calculate and store an intensity ratio (GRI 305-4).
#[tauri::command]
pub fn calculate_intensity(
    db: State<Database>,
    period_id: i64,
    input: IntensityInput,
) -> Result<intensity::IntensityResult> {
    let conn = db.0.lock().unwrap();
    Ok(intensity::calculate_and_store(
        &conn,
        period_id,
        input.includes_scope1,
        input.includes_scope2,
        input.includes_scope3,
        &input.metric_name,
        input.metric_value,
        &input.metric_unit,
        input.scope1_tco2e,
        input.scope2_location_tco2e,
        input.scope3_tco2e,
    )?)
}

/// Save an intensity metric (GRI 305-4) — auto-calculates from current period inventory.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn save_intensity_metric(
    db: State<Database>,
    period_id: i64,
    metric_name: String,
    metric_value: f64,
    metric_unit: String,
    includes_scope1: bool,
    includes_scope2: bool,
    includes_scope3: bool,
) -> Result<intensity::IntensityResult> {
    let conn = db.0.lock().unwrap();
    let inv = engine::calculate_period(&conn, period_id)?;
    Ok(intensity::calculate_and_store(
        &conn,
        period_id,
        includes_scope1,
        includes_scope2,
        includes_scope3,
        &metric_name,
        metric_value,
        &metric_unit,
        inv.scope1.gross_tco2e,
        inv.scope2.location_based_tco2e,
        inv.scope3.gross_tco2e,
    )?)
}

/// List all stored intensity results for a period.
#[tauri::command]
pub fn list_intensity_results(
    db: State<Database>,
    period_id: i64,
) -> Result<Vec<intensity::IntensityResult>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT includes_scope1, includes_scope2, includes_scope3, total_emissions_tco2e,
                metric_name, metric_value, metric_unit, intensity_ratio, scope3_intensity_ratio
         FROM intensity_ratios WHERE period_id = ?1 ORDER BY created_at"
    )?;
    let rows = stmt.query_map(params![period_id], |r| {
        Ok(intensity::IntensityResult {
            includes_scope1: r.get::<_, i64>(0)? != 0,
            includes_scope2: r.get::<_, i64>(1)? != 0,
            includes_scope3: r.get::<_, i64>(2)? != 0,
            total_emissions_tco2e: r.get(3)?,
            metric_name: r.get(4)?,
            metric_value: r.get(5)?,
            metric_unit: r.get(6)?,
            intensity_ratio: r.get(7)?,
            scope3_intensity_ratio: r.get(8)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn delete_intensity_result(db: State<Database>, period_id: i64, metric_name: String) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM intensity_ratios WHERE period_id = ?1 AND metric_name = ?2",
        params![period_id, metric_name],
    )?;
    Ok(())
}

/// List all GWP values for a given AR version.
#[tauri::command]
pub fn list_gwp_values(
    db: State<Database>,
    ar_version: String,
) -> Result<Vec<serde_json::Value>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT gas, ar_version, gwp_100, notes FROM gwp_values WHERE ar_version = ?1 ORDER BY gas"
    )?;
    let values: Vec<serde_json::Value> = stmt.query_map(params![ar_version], |row| {
        Ok(serde_json::json!({
            "gas": row.get::<_, String>(0)?,
            "ar_version": row.get::<_, String>(1)?,
            "gwp_100": row.get::<_, f64>(2)?,
            "notes": row.get::<_, Option<String>>(3)?,
        }))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(values)
}

/// Get the audit log for a record.
#[tauri::command]
pub fn get_audit_log(
    db: State<Database>,
    table_name: String,
    record_id: i64,
) -> Result<Vec<serde_json::Value>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, action, field_name, old_value, new_value, user_id, timestamp, reason
         FROM audit_log WHERE table_name = ?1 AND record_id = ?2 ORDER BY timestamp DESC"
    )?;
    let entries: Vec<serde_json::Value> = stmt.query_map(params![table_name, record_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "action": row.get::<_, String>(1)?,
            "field_name": row.get::<_, Option<String>>(2)?,
            "old_value": row.get::<_, Option<String>>(3)?,
            "new_value": row.get::<_, Option<String>>(4)?,
            "user_id": row.get::<_, Option<String>>(5)?,
            "timestamp": row.get::<_, i64>(6)?,
            "reason": row.get::<_, Option<String>>(7)?,
        }))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entries)
}
