use crate::db::Database;
use crate::error::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmissionSource {
    pub id: i64,
    pub entity_id: i64,
    pub period_id: i64,
    pub scope: i64,
    pub scope2_method: Option<String>,
    pub scope3_category: Option<i64>,
    pub category_name: String,
    pub ghg_type: String,
    pub activity_value: f64,
    pub activity_unit: String,
    pub activity_source: Option<String>,
    pub emission_factor_value: f64,
    pub emission_factor_unit: String,
    pub emission_factor_source: String,
    pub emission_factor_citation: Option<String>,
    pub gwp_value: f64,
    pub emissions_tco2e: Option<f64>,
    pub biogenic_co2_tco2e: Option<f64>,
    pub uncertainty_pct: Option<f64>,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSourceInput {
    pub entity_id: i64,
    pub period_id: i64,
    pub scope: i64,
    pub scope2_method: Option<String>,
    pub scope3_category: Option<i64>,
    pub category_name: String,
    pub ghg_type: String,
    pub activity_value: f64,
    pub activity_unit: String,
    pub activity_source: Option<String>,
    pub emission_factor_value: f64,
    pub emission_factor_unit: String,
    pub emission_factor_source: String,
    pub emission_factor_citation: Option<String>,
    pub gwp_value: f64,
    pub biogenic_co2_tco2e: Option<f64>,
    pub uncertainty_pct: Option<f64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn create_source(db: State<Database>, input: CreateSourceInput) -> Result<EmissionSource> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO emission_sources (
           entity_id, period_id, scope, scope2_method, scope3_category,
           category_name, ghg_type, activity_value, activity_unit, activity_source,
           emission_factor_value, emission_factor_unit, emission_factor_source,
           emission_factor_citation, gwp_value, biogenic_co2_tco2e, uncertainty_pct, notes
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        params![
            input.entity_id, input.period_id, input.scope,
            input.scope2_method, input.scope3_category, input.category_name,
            input.ghg_type, input.activity_value, input.activity_unit,
            input.activity_source, input.emission_factor_value,
            input.emission_factor_unit, input.emission_factor_source,
            input.emission_factor_citation, input.gwp_value,
            input.biogenic_co2_tco2e, input.uncertainty_pct, input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();

    // Audit log
    conn.execute(
        "INSERT INTO audit_log (table_name, record_id, action, user_id) VALUES ('emission_sources', ?1, 'INSERT', 'local')",
        params![id],
    )?;

    get_source_inner(&conn, id)
}

#[tauri::command]
pub fn update_source(
    db: State<Database>,
    id: i64,
    input: CreateSourceInput,
    reason: Option<String>,
) -> Result<EmissionSource> {
    let conn = db.0.lock().unwrap();

    // Get old values for audit log
    let old: EmissionSource = get_source_inner(&conn, id)?;

    conn.execute(
        "UPDATE emission_sources SET
           entity_id=?1, scope=?2, scope2_method=?3, scope3_category=?4,
           category_name=?5, ghg_type=?6, activity_value=?7, activity_unit=?8,
           activity_source=?9, emission_factor_value=?10, emission_factor_unit=?11,
           emission_factor_source=?12, emission_factor_citation=?13, gwp_value=?14,
           biogenic_co2_tco2e=?15, uncertainty_pct=?16, notes=?17,
           emissions_tco2e=NULL, updated_at=unixepoch()
         WHERE id=?18",
        params![
            input.entity_id, input.scope, input.scope2_method, input.scope3_category,
            input.category_name, input.ghg_type, input.activity_value, input.activity_unit,
            input.activity_source, input.emission_factor_value, input.emission_factor_unit,
            input.emission_factor_source, input.emission_factor_citation, input.gwp_value,
            input.biogenic_co2_tco2e, input.uncertainty_pct, input.notes, id,
        ],
    )?;

    // Audit log with old/new activity values
    conn.execute(
        "INSERT INTO audit_log (table_name, record_id, action, field_name, old_value, new_value, reason, user_id)
         VALUES ('emission_sources', ?1, 'UPDATE', 'activity_value', ?2, ?3, ?4, 'local')",
        params![id, old.activity_value.to_string(), input.activity_value.to_string(), reason],
    )?;

    get_source_inner(&conn, id)
}

#[tauri::command]
pub fn delete_source(db: State<Database>, id: i64, reason: Option<String>) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO audit_log (table_name, record_id, action, reason, user_id)
         VALUES ('emission_sources', ?1, 'DELETE', ?2, 'local')",
        params![id, reason],
    )?;
    conn.execute("DELETE FROM emission_sources WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn list_sources(db: State<Database>, period_id: i64, scope: Option<i64>) -> Result<Vec<EmissionSource>> {
    let conn = db.0.lock().unwrap();
    let sql = match scope {
        Some(_) => "SELECT id, entity_id, period_id, scope, scope2_method, scope3_category,
                category_name, ghg_type, activity_value, activity_unit, activity_source,
                emission_factor_value, emission_factor_unit, emission_factor_source,
                emission_factor_citation, gwp_value, emissions_tco2e, biogenic_co2_tco2e,
                uncertainty_pct, is_excluded, exclusion_reason, notes
           FROM emission_sources WHERE period_id = ?1 AND scope = ?2 ORDER BY category_name",
        None => "SELECT id, entity_id, period_id, scope, scope2_method, scope3_category,
                category_name, ghg_type, activity_value, activity_unit, activity_source,
                emission_factor_value, emission_factor_unit, emission_factor_source,
                emission_factor_citation, gwp_value, emissions_tco2e, biogenic_co2_tco2e,
                uncertainty_pct, is_excluded, exclusion_reason, notes
           FROM emission_sources WHERE period_id = ?1 ORDER BY scope, category_name",
    };

    let conn_ref = &conn;
    let sources = if let Some(s) = scope {
        let mut stmt = conn_ref.prepare(sql)?;
        let rows = stmt.query_map(params![period_id, s], map_source_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let mut stmt = conn_ref.prepare(sql)?;
        let rows = stmt.query_map(params![period_id], map_source_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    Ok(sources)
}

#[tauri::command]
pub fn list_emission_factors(
    db: State<Database>,
    category: Option<String>,
    region: Option<String>,
) -> Result<Vec<serde_json::Value>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, subcategory, region, ghg_type,
                factor_value, factor_unit, ar_version, source, valid_from
         FROM emission_factors
         WHERE (?1 IS NULL OR category = ?1) AND (?2 IS NULL OR region = ?2)
         ORDER BY category, name LIMIT 500"
    )?;
    let factors: Vec<serde_json::Value> = stmt.query_map(params![category, region], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "category": row.get::<_, String>(2)?,
            "subcategory": row.get::<_, Option<String>>(3)?,
            "region": row.get::<_, Option<String>>(4)?,
            "ghg_type": row.get::<_, String>(5)?,
            "factor_value": row.get::<_, f64>(6)?,
            "factor_unit": row.get::<_, String>(7)?,
            "ar_version": row.get::<_, Option<String>>(8)?,
            "source": row.get::<_, String>(9)?,
            "valid_from": row.get::<_, Option<String>>(10)?,
        }))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(factors)
}

fn get_source_inner(conn: &rusqlite::Connection, id: i64) -> Result<EmissionSource> {
    Ok(conn.query_row(
        "SELECT id, entity_id, period_id, scope, scope2_method, scope3_category,
                category_name, ghg_type, activity_value, activity_unit, activity_source,
                emission_factor_value, emission_factor_unit, emission_factor_source,
                emission_factor_citation, gwp_value, emissions_tco2e, biogenic_co2_tco2e,
                uncertainty_pct, is_excluded, exclusion_reason, notes
         FROM emission_sources WHERE id = ?1",
        params![id],
        map_source_row,
    )?)
}

fn map_source_row(row: &rusqlite::Row) -> rusqlite::Result<EmissionSource> {
    Ok(EmissionSource {
        id: row.get(0)?,
        entity_id: row.get(1)?,
        period_id: row.get(2)?,
        scope: row.get(3)?,
        scope2_method: row.get(4)?,
        scope3_category: row.get(5)?,
        category_name: row.get(6)?,
        ghg_type: row.get(7)?,
        activity_value: row.get(8)?,
        activity_unit: row.get(9)?,
        activity_source: row.get(10)?,
        emission_factor_value: row.get(11)?,
        emission_factor_unit: row.get(12)?,
        emission_factor_source: row.get(13)?,
        emission_factor_citation: row.get(14)?,
        gwp_value: row.get(15)?,
        emissions_tco2e: row.get(16)?,
        biogenic_co2_tco2e: row.get(17)?,
        uncertainty_pct: row.get(18)?,
        is_excluded: row.get::<_, i64>(19)? != 0,
        exclusion_reason: row.get(20)?,
        notes: row.get(21)?,
    })
}
