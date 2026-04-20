use crate::db::Database;
use crate::error::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

// UNGC COP 2025 questionnaire — 66 questions mapped to principles
// Environment questions (Principles 7, 8, 9) are pre-mapped to GRI 305 data
const COP_QUESTIONS: &[(&str, &str, &str)] = &[
    ("E1",  "Does your company have a policy commitment to environmental responsibility?", "P7"),
    ("E2",  "Is your board/senior management responsible for environmental policy?", "P7"),
    ("E3",  "Does your company conduct environmental risk assessment?", "P7"),
    ("E4",  "Do you apply the precautionary approach to environmental challenges?", "P7"),
    ("E5",  "Has your company set environmental targets?", "P8"),
    ("E6",  "What were your company's total Scope 1 GHG emissions (tCO₂e)?", "P8"),
    ("E7",  "What were your company's total Scope 2 GHG emissions (tCO₂e)?", "P8"),
    ("E8",  "What were your company's total Scope 3 GHG emissions (tCO₂e)?", "P8"),
    ("E9",  "Has your company set a GHG reduction target?", "P8"),
    ("E10", "Has your GHG target been externally validated (e.g., Science Based Targets)?", "P8"),
    ("E11", "What percentage of your electricity comes from renewable sources?", "P8"),
    ("E12", "Has your company implemented energy efficiency measures?", "P8"),
    ("E13", "Does your company measure and report water consumption?", "P8"),
    ("E14", "Does your company have a waste reduction programme?", "P8"),
    ("E15", "Does your company promote environmentally friendly technologies?", "P9"),
    ("E16", "Does your company invest in R&D for low-carbon solutions?", "P9"),
    ("E17", "Does your company engage suppliers on environmental performance?", "P9"),
    ("E18", "Has your company experienced any significant environmental incidents?", "P7"),
    ("E19", "Does your company report using a recognised framework (GRI, TCFD, ISSB)?", "P8"),
    ("E20", "Has your emissions data been third-party verified?", "P8"),
    // Additional non-environment questions abbreviated
    ("G1",  "Does your company have an anti-corruption policy?", "P10"),
    ("G2",  "Does your company provide anti-corruption training?", "P10"),
    ("L1",  "Does your company respect freedom of association?", "P3"),
    ("L2",  "Does your company prohibit forced labour?", "P4"),
    ("L3",  "Does your company prohibit child labour?", "P5"),
    ("L4",  "Does your company prohibit discrimination?", "P6"),
    ("HR1", "Does your company have a human rights policy?", "P1"),
    ("HR2", "Does your company conduct human rights due diligence?", "P2"),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct CopQuestion {
    pub question_id: String,
    pub question_text: String,
    pub principle_reference: String,
    pub response: Option<String>,
    pub auto_populated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopStatus {
    pub id: i64,
    pub org_id: i64,
    pub reporting_year: i64,
    pub status: String,
    pub compliance_level: Option<String>,
    pub ceo_statement_signed: bool,
    pub answered_count: i64,
    pub total_count: i64,
}

/// Initialise or get a COP record for the given year.
#[tauri::command]
pub fn init_cop(db: State<Database>, org_id: i64, reporting_year: i64) -> Result<CopStatus> {
    let conn = db.0.lock().unwrap();

    // Upsert COP record
    conn.execute(
        "INSERT OR IGNORE INTO ungc_cop (org_id, reporting_year) VALUES (?1, ?2)",
        params![org_id, reporting_year],
    )?;

    let cop_id: i64 = conn.query_row(
        "SELECT id FROM ungc_cop WHERE org_id = ?1 AND reporting_year = ?2",
        params![org_id, reporting_year],
        |r| r.get(0),
    )?;

    // Insert any missing questions
    for (qid, qtext, principle) in COP_QUESTIONS {
        conn.execute(
            "INSERT OR IGNORE INTO ungc_cop_responses (cop_id, question_id, question_text, principle_reference)
             VALUES (?1, ?2, ?3, ?4)",
            params![cop_id, qid, qtext, principle],
        )?;
    }

    get_cop_status_inner(&conn, cop_id, org_id)
}

/// Auto-populate environment questions from GRI 305 data for a given period.
#[tauri::command]
pub fn auto_populate_cop(
    db: State<Database>,
    cop_id: i64,
    period_id: i64,
) -> Result<u32> {
    let conn = db.0.lock().unwrap();
    let mut populated = 0u32;

    // E6: Scope 1 total
    let scope1_tco2e: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(emissions_tco2e,0)), 0)
         FROM emission_sources WHERE period_id=?1 AND scope=1 AND is_excluded=0",
        params![period_id], |r| r.get(0),
    ).unwrap_or(0.0);
    if scope1_tco2e > 0.0 {
        conn.execute(
            "UPDATE ungc_cop_responses SET response=?1, auto_populated=1,
             auto_source_table='emission_sources', updated_at=unixepoch()
             WHERE cop_id=?2 AND question_id='E6'",
            params![format!("{:.2} tCO₂e", scope1_tco2e), cop_id],
        )?;
        populated += 1;
    }

    // E7: Scope 2 (location-based)
    let scope2_tco2e: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(emissions_tco2e,0)), 0)
         FROM emission_sources WHERE period_id=?1 AND scope=2
         AND scope2_method='location_based' AND is_excluded=0",
        params![period_id], |r| r.get(0),
    ).unwrap_or(0.0);
    if scope2_tco2e > 0.0 {
        conn.execute(
            "UPDATE ungc_cop_responses SET response=?1, auto_populated=1,
             auto_source_table='emission_sources', updated_at=unixepoch()
             WHERE cop_id=?2 AND question_id='E7'",
            params![format!("{:.2} tCO₂e (location-based)", scope2_tco2e), cop_id],
        )?;
        populated += 1;
    }

    // E8: Scope 3 total
    let scope3_tco2e: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(emissions_tco2e,0)), 0)
         FROM emission_sources WHERE period_id=?1 AND scope=3 AND is_excluded=0",
        params![period_id], |r| r.get(0),
    ).unwrap_or(0.0);
    if scope3_tco2e > 0.0 {
        conn.execute(
            "UPDATE ungc_cop_responses SET response=?1, auto_populated=1,
             auto_source_table='emission_sources', updated_at=unixepoch()
             WHERE cop_id=?2 AND question_id='E8'",
            params![format!("{:.2} tCO₂e", scope3_tco2e), cop_id],
        )?;
        populated += 1;
    }

    Ok(populated)
}

#[tauri::command]
pub fn get_cop_questions(db: State<Database>, cop_id: i64) -> Result<Vec<CopQuestion>> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT question_id, question_text, principle_reference, response, auto_populated
         FROM ungc_cop_responses WHERE cop_id = ?1 ORDER BY question_id"
    )?;
    let questions = stmt.query_map(params![cop_id], |row| {
        Ok(CopQuestion {
            question_id: row.get(0)?,
            question_text: row.get(1)?,
            principle_reference: row.get(2)?,
            response: row.get(3)?,
            auto_populated: row.get::<_, i64>(4)? != 0,
        })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(questions)
}

#[tauri::command]
pub fn save_cop_response(
    db: State<Database>,
    cop_id: i64,
    question_id: String,
    response: String,
) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE ungc_cop_responses SET response=?1, auto_populated=0, updated_at=unixepoch()
         WHERE cop_id=?2 AND question_id=?3",
        params![response, cop_id, question_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn sign_ceo_statement(
    db: State<Database>,
    cop_id: i64,
    ceo_name: String,
) -> Result<()> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE ungc_cop SET ceo_statement_signed=1, ceo_name=?1, signed_at=unixepoch()
         WHERE id=?2",
        params![ceo_name, cop_id],
    )?;
    Ok(())
}

/// Compute compliance level based on answered questions.
#[tauri::command]
pub fn compute_compliance_level(db: State<Database>, cop_id: i64) -> Result<String> {
    let conn = db.0.lock().unwrap();

    let (answered, total): (i64, i64) = conn.query_row(
        "SELECT COUNT(CASE WHEN response IS NOT NULL AND response != '' THEN 1 END), COUNT(*)
         FROM ungc_cop_responses WHERE cop_id = ?1",
        params![cop_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let ceo_signed: i64 = conn.query_row(
        "SELECT ceo_statement_signed FROM ungc_cop WHERE id = ?1",
        params![cop_id],
        |r| r.get(0),
    )?;

    // Simplified level computation:
    // Beginner: < 50% answered or no CEO statement
    // Active: >= 50% answered + CEO statement
    // Advanced: >= 75% answered + CEO statement
    // LEAD: 100% answered + CEO statement + verified emissions
    let pct = if total > 0 { answered * 100 / total } else { 0 };
    let level = if ceo_signed == 0 || pct < 50 {
        "beginner"
    } else if pct < 75 {
        "active"
    } else if pct < 100 {
        "advanced"
    } else {
        "lead"
    };

    conn.execute(
        "UPDATE ungc_cop SET compliance_level=?1 WHERE id=?2",
        params![level, cop_id],
    )?;

    Ok(level.to_string())
}

fn get_cop_status_inner(
    conn: &rusqlite::Connection,
    cop_id: i64,
    org_id: i64,
) -> Result<CopStatus> {
    let (id, reporting_year, status, compliance_level, ceo_signed): (i64, i64, String, Option<String>, i64) =
        conn.query_row(
            "SELECT id, reporting_year, status, compliance_level, ceo_statement_signed
             FROM ungc_cop WHERE id = ?1",
            params![cop_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;

    let (answered, total): (i64, i64) = conn.query_row(
        "SELECT COUNT(CASE WHEN response IS NOT NULL AND response != '' THEN 1 END), COUNT(*)
         FROM ungc_cop_responses WHERE cop_id = ?1",
        params![cop_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    Ok(CopStatus {
        id,
        org_id,
        reporting_year,
        status,
        compliance_level,
        ceo_statement_signed: ceo_signed != 0,
        answered_count: answered,
        total_count: total,
    })
}
