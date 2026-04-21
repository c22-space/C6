use crate::db::Database;
use crate::engine;
use crate::error::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct GriReport {
    pub period_year: i64,
    pub org_name: String,
    pub boundary_method: String,
    pub gwp_ar_version: String,
    pub disclosure_305_1: Disclosure3051,
    pub disclosure_305_2: Disclosure3052,
    pub disclosure_305_3: Disclosure3053,
    pub disclosure_305_4: Option<Disclosure3054>,
    pub disclosure_305_5: Option<Disclosure3055>,
    pub disclosure_305_6: Vec<serde_json::Value>,
    pub disclosure_305_7: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disclosure3051 {
    pub gross_tco2e: f64,
    pub biogenic_co2_tco2e: f64,
    pub by_gas: std::collections::HashMap<String, f64>,
    pub gases_included: Vec<String>,
    pub consolidation_approach: String,
    pub uncertainty_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disclosure3052 {
    pub location_based_tco2e: f64,
    pub market_based_tco2e: f64,
    pub contractual_coverage_pct: f64,
    pub consolidation_approach: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disclosure3053 {
    pub gross_tco2e: f64,
    pub upstream_tco2e: f64,
    pub downstream_tco2e: f64,
    pub categories: Vec<serde_json::Value>,
    pub excluded_categories: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disclosure3054 {
    pub intensity_ratio: f64,
    pub metric_name: String,
    pub metric_unit: String,
    pub scopes_included: String,
    pub scope3_intensity_ratio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disclosure3055 {
    pub reduction_tco2e: f64,
    pub reduction_pct: f64,
    pub baseline_year: i64,
    pub methodology: String,
}

/// Generate a full GRI 305 disclosure report for a period.
#[tauri::command]
pub fn generate_gri305_report(
    db: State<Database>,
    period_id: i64,
) -> Result<GriReport> {
    let conn = db.0.lock().unwrap();

    let (org_name, boundary_method, period_year, gwp_ar): (String, String, i64, String) =
        conn.query_row(
            "SELECT o.name, o.boundary_method, rp.year, rp.gwp_ar_version
             FROM reporting_periods rp
             JOIN organizations o ON rp.org_id = o.id
             WHERE rp.id = ?1",
            params![period_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;

    let inventory = engine::calculate_period(&conn, period_id)?;

    // 305-1
    let gases_included: Vec<String> = inventory.scope1.by_gas.keys().cloned().collect();
    let d1 = Disclosure3051 {
        gross_tco2e: inventory.scope1.gross_tco2e,
        biogenic_co2_tco2e: inventory.scope1.biogenic_co2_tco2e,
        by_gas: inventory.scope1.by_gas.clone(),
        gases_included,
        consolidation_approach: boundary_method.clone(),
        uncertainty_pct: inventory.scope1.combined_uncertainty_pct,
    };

    // 305-2
    let d2 = Disclosure3052 {
        location_based_tco2e: inventory.scope2.location_based_tco2e,
        market_based_tco2e: inventory.scope2.market_based_tco2e,
        contractual_coverage_pct: inventory.scope2.contractual_coverage_pct,
        consolidation_approach: boundary_method.clone(),
    };

    // 305-3
    let cat_values: Vec<serde_json::Value> = inventory.scope3.categories.iter().map(|c| {
        serde_json::json!({
            "category": c.category,
            "name": c.category_name,
            "direction": c.direction,
            "tco2e": c.total_tco2e,
            "excluded": c.is_excluded,
            "exclusion_reason": c.exclusion_reason,
        })
    }).collect();

    let d3 = Disclosure3053 {
        gross_tco2e: inventory.scope3.gross_tco2e,
        upstream_tco2e: inventory.scope3.upstream_tco2e,
        downstream_tco2e: inventory.scope3.downstream_tco2e,
        categories: cat_values,
        excluded_categories: inventory.scope3.excluded_categories.clone(),
    };

    // 305-4: Intensity (if exists)
    let d4: Option<Disclosure3054> = conn.query_row(
        "SELECT intensity_ratio, metric_name, metric_unit,
                includes_scope1, includes_scope2, includes_scope3, scope3_intensity_ratio
         FROM intensity_ratios WHERE period_id = ?1 LIMIT 1",
        params![period_id],
        |r| {
            let s1: i64 = r.get(3)?;
            let s2: i64 = r.get(4)?;
            let s3: i64 = r.get(5)?;
            let scopes: Vec<&str> = [
                if s1 != 0 { Some("Scope 1") } else { None },
                if s2 != 0 { Some("Scope 2") } else { None },
                if s3 != 0 { Some("Scope 3") } else { None },
            ].iter().flatten().cloned().collect();
            Ok(Disclosure3054 {
                intensity_ratio: r.get(0)?,
                metric_name: r.get(1)?,
                metric_unit: r.get(2)?,
                scopes_included: scopes.join(" + "),
                scope3_intensity_ratio: r.get(6)?,
            })
        },
    ).ok();

    // 305-5: Reductions (if exists)
    let d5: Option<Disclosure3055> = conn.query_row(
        "SELECT reduction_tco2e, reduction_pct, baseline_year, methodology
         FROM reductions WHERE period_id = ?1 LIMIT 1",
        params![period_id],
        |r| Ok(Disclosure3055 {
            reduction_tco2e: r.get(0)?,
            reduction_pct: r.get(1)?,
            baseline_year: r.get(2)?,
            methodology: r.get(3)?,
        }),
    ).ok();

    // 305-6: ODS
    let ods: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare(
            "SELECT substance, production_metric_tons, imports_metric_tons,
                    exports_metric_tons, cfc11_equivalent
             FROM ods_emissions WHERE period_id = ?1"
        )?;
        let rows = stmt.query_map(params![period_id], |r| {
            Ok(serde_json::json!({
                "substance": r.get::<_, String>(0)?,
                "production_mt": r.get::<_, f64>(1)?,
                "imports_mt": r.get::<_, f64>(2)?,
                "exports_mt": r.get::<_, f64>(3)?,
                "cfc11_equivalent": r.get::<_, f64>(4)?,
            }))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    // 305-7: Air emissions
    let air: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare(
            "SELECT emission_type, substance, value_metric_tons, measurement_method
             FROM air_emissions WHERE period_id = ?1"
        )?;
        let rows = stmt.query_map(params![period_id], |r| {
            Ok(serde_json::json!({
                "type": r.get::<_, String>(0)?,
                "substance": r.get::<_, Option<String>>(1)?,
                "value_mt": r.get::<_, f64>(2)?,
                "method": r.get::<_, String>(3)?,
            }))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    Ok(GriReport {
        period_year,
        org_name,
        boundary_method,
        gwp_ar_version: gwp_ar,
        disclosure_305_1: d1,
        disclosure_305_2: d2,
        disclosure_305_3: d3,
        disclosure_305_4: d4,
        disclosure_305_5: d5,
        disclosure_305_6: ods,
        disclosure_305_7: air,
    })
}

// ── GRI 305-5: Reductions ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Reduction {
    pub id: i64,
    pub period_id: i64,
    pub baseline_year: i64,
    pub baseline_tco2e: f64,
    pub current_tco2e: f64,
    pub reduction_tco2e: f64,
    pub reduction_pct: f64,
    pub methodology: String,
}

#[tauri::command]
pub fn list_reductions(db: State<Database>, period_id: i64) -> Result<Vec<Reduction>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, period_id, baseline_year, baseline_tco2e, current_tco2e,
                reduction_tco2e, reduction_pct, methodology
         FROM reductions WHERE period_id = ?1 ORDER BY created_at"
    )?;
    let rows = stmt.query_map(params![period_id], |r| {
        Ok(Reduction {
            id: r.get(0)?,
            period_id: r.get(1)?,
            baseline_year: r.get(2)?,
            baseline_tco2e: r.get(3)?,
            current_tco2e: r.get(4)?,
            reduction_tco2e: r.get(5)?,
            reduction_pct: r.get(6)?,
            methodology: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn create_reduction(
    db: State<Database>,
    period_id: i64,
    baseline_year: i64,
    baseline_tco2e: f64,
    current_tco2e: f64,
    methodology: String,
) -> Result<Reduction> {
    let conn = db.0.lock().unwrap();
    let reduction_tco2e = (baseline_tco2e - current_tco2e).max(0.0);
    let reduction_pct = if baseline_tco2e > 0.0 {
        (reduction_tco2e / baseline_tco2e) * 100.0
    } else {
        0.0
    };
    conn.execute(
        "INSERT INTO reductions (period_id, baseline_year, baseline_tco2e, current_tco2e,
         reduction_tco2e, reduction_pct, methodology)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![period_id, baseline_year, baseline_tco2e, current_tco2e,
                reduction_tco2e, reduction_pct, methodology],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Reduction { id, period_id, baseline_year, baseline_tco2e, current_tco2e, reduction_tco2e, reduction_pct, methodology })
}

#[tauri::command]
pub fn delete_reduction(db: State<Database>, id: i64) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM reductions WHERE id = ?1", params![id])?;
    Ok(())
}

// ── GRI 305-6: ODS Emissions ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct OdsEntry {
    pub id: i64,
    pub period_id: i64,
    pub substance: String,
    pub production_metric_tons: f64,
    pub imports_metric_tons: f64,
    pub exports_metric_tons: f64,
    pub cfc11_equivalent: f64,
}

#[tauri::command]
pub fn list_ods_emissions(db: State<Database>, period_id: i64) -> Result<Vec<OdsEntry>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, period_id, substance, production_metric_tons, imports_metric_tons,
                exports_metric_tons, cfc11_equivalent
         FROM ods_emissions WHERE period_id = ?1 ORDER BY substance"
    )?;
    let rows = stmt.query_map(params![period_id], |r| {
        Ok(OdsEntry {
            id: r.get(0)?,
            period_id: r.get(1)?,
            substance: r.get(2)?,
            production_metric_tons: r.get(3)?,
            imports_metric_tons: r.get(4)?,
            exports_metric_tons: r.get(5)?,
            cfc11_equivalent: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn create_ods_emission(
    db: State<Database>,
    period_id: i64,
    substance: String,
    production_metric_tons: f64,
    imports_metric_tons: f64,
    exports_metric_tons: f64,
    cfc11_equivalent: f64,
) -> Result<OdsEntry> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO ods_emissions (period_id, substance, production_metric_tons,
         imports_metric_tons, exports_metric_tons, cfc11_equivalent)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![period_id, substance, production_metric_tons, imports_metric_tons,
                exports_metric_tons, cfc11_equivalent],
    )?;
    let id = conn.last_insert_rowid();
    Ok(OdsEntry { id, period_id, substance, production_metric_tons, imports_metric_tons, exports_metric_tons, cfc11_equivalent })
}

#[tauri::command]
pub fn delete_ods_emission(db: State<Database>, id: i64) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM ods_emissions WHERE id = ?1", params![id])?;
    Ok(())
}

// ── GRI 305-7: Air Emissions ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AirEntry {
    pub id: i64,
    pub period_id: i64,
    pub emission_type: String,
    pub substance: Option<String>,
    pub value_metric_tons: f64,
    pub measurement_method: String,
}

#[tauri::command]
pub fn list_air_emissions(db: State<Database>, period_id: i64) -> Result<Vec<AirEntry>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, period_id, emission_type, substance, value_metric_tons, measurement_method
         FROM air_emissions WHERE period_id = ?1 ORDER BY emission_type"
    )?;
    let rows = stmt.query_map(params![period_id], |r| {
        Ok(AirEntry {
            id: r.get(0)?,
            period_id: r.get(1)?,
            emission_type: r.get(2)?,
            substance: r.get(3)?,
            value_metric_tons: r.get(4)?,
            measurement_method: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn create_air_emission(
    db: State<Database>,
    period_id: i64,
    emission_type: String,
    substance: Option<String>,
    value_metric_tons: f64,
    measurement_method: String,
) -> Result<AirEntry> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO air_emissions (period_id, emission_type, substance, value_metric_tons, measurement_method)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![period_id, emission_type, substance, value_metric_tons, measurement_method],
    )?;
    let id = conn.last_insert_rowid();
    Ok(AirEntry { id, period_id, emission_type, substance, value_metric_tons, measurement_method })
}

#[tauri::command]
pub fn delete_air_emission(db: State<Database>, id: i64) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM air_emissions WHERE id = ?1", params![id])?;
    Ok(())
}

/// Export a CSV of all emission sources for a period.
#[tauri::command]
pub fn export_sources_csv(
    db: State<Database>,
    period_id: i64,
    path: String,
) -> Result<()> {
    let conn = db.0.lock().unwrap();
    let mut wtr = csv::Writer::from_path(&path)
        .map_err(|e| crate::error::Error::App(e.to_string()))?;

    wtr.write_record([
        "scope","scope2_method","scope3_category","category_name","ghg_type",
        "activity_value","activity_unit","activity_source",
        "emission_factor_value","emission_factor_unit","emission_factor_source",
        "gwp_value","emissions_tco2e","biogenic_co2_tco2e",
        "uncertainty_pct","is_excluded","exclusion_reason","notes",
    ]).map_err(|e| crate::error::Error::App(e.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT scope, scope2_method, scope3_category, category_name, ghg_type,
                activity_value, activity_unit, activity_source,
                emission_factor_value, emission_factor_unit, emission_factor_source,
                gwp_value, emissions_tco2e, biogenic_co2_tco2e,
                uncertainty_pct, is_excluded, exclusion_reason, notes
         FROM emission_sources WHERE period_id = ?1 ORDER BY scope, category_name"
    )?;

    stmt.query_map(params![period_id], |r| {
        Ok((
            r.get::<_, i64>(0)?.to_string(),
            r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            r.get::<_, Option<i64>>(2)?.map(|v| v.to_string()).unwrap_or_default(),
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, f64>(5)?.to_string(),
            r.get::<_, String>(6)?,
            r.get::<_, Option<String>>(7)?.unwrap_or_default(),
            r.get::<_, f64>(8)?.to_string(),
            r.get::<_, String>(9)?,
            r.get::<_, String>(10)?,
            r.get::<_, f64>(11)?.to_string(),
            r.get::<_, Option<f64>>(12)?.map(|v| v.to_string()).unwrap_or_default(),
            r.get::<_, Option<f64>>(13)?.map(|v| v.to_string()).unwrap_or_default(),
            r.get::<_, Option<f64>>(14)?.map(|v| v.to_string()).unwrap_or_default(),
            r.get::<_, i64>(15)?.to_string(),
            r.get::<_, Option<String>>(16)?.unwrap_or_default(),
            r.get::<_, Option<String>>(17)?.unwrap_or_default(),
        ))
    })?.try_for_each(|row| {
        let r = row?;
        wtr.write_record([
            &r.0,&r.1,&r.2,&r.3,&r.4,&r.5,&r.6,&r.7,
            &r.8,&r.9,&r.10,&r.11,&r.12,&r.13,&r.14,&r.15,&r.16,&r.17,
        ]).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })?;

    wtr.flush().map_err(|e| crate::error::Error::App(e.to_string()))?;
    Ok(())
}
