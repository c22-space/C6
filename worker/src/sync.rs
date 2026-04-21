use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, Env, Request, Response, Result};

use crate::jwt::{self, Claims};

#[derive(Serialize, Deserialize)]
struct SyncRecord {
    table: String,
    id: i64,
    data: serde_json::Map<String, serde_json::Value>,
    deleted: bool,
    updated_at: i64,
}

#[derive(Serialize, Deserialize)]
struct EnterpriseUser {
    id: String,
    email: String,
    role: String,
    is_active: i64,
    invited_at: i64,
    last_seen: Option<i64>,
}

const SYNC_TABLES: &[&str] = &[
    "emission_sources",
    "reporting_periods",
    "entities",
    "ungc_cop",
    "ungc_cop_responses",
];

pub async fn handle_pull(req: Request, env: &Env) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    let url = req.url()?;
    let since: i64 = url
        .query_pairs()
        .find(|(k, _)| k == "since")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);

    let db = env.d1("DB")?;
    let mut changes: Vec<serde_json::Value> = Vec::new();

    for table in SYNC_TABLES {
        let rows = db
            .prepare(&format!(
                "SELECT * FROM {} WHERE org_id = ?1 AND updated_at > ?2 \
                 ORDER BY updated_at ASC LIMIT 500",
                table
            ))
            .bind(&[
                JsValue::from_str(&claims.org_id),
                JsValue::from_f64(since as f64),
            ])?
            .all()
            .await?
            .results::<serde_json::Value>()?;

        for row in rows {
            let deleted = row
                .get("deleted_at")
                .map(|v| !v.is_null())
                .unwrap_or(false);
            let updated_at = row
                .get("updated_at")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            changes.push(serde_json::json!({
                "table": table,
                "id": row.get("id"),
                "data": row,
                "deleted": deleted,
                "updated_at": updated_at,
            }));
        }
    }

    let server_time = jwt::now();
    Response::from_json(&serde_json::json!({ "changes": changes, "server_time": server_time }))
}

pub async fn handle_push(mut req: Request, env: &Env) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    if claims.role == "viewer" {
        return Response::error("Viewers cannot write data", 403);
    }

    let records: Vec<SyncRecord> =
        req.json().await.map_err(|_| "Invalid JSON")?;

    let db = env.d1("DB")?;
    let now = jwt::now();
    let mut synced = 0usize;

    for mut record in records {
        if !SYNC_TABLES.contains(&record.table.as_str()) {
            continue;
        }

        record.data.insert("org_id".into(), claims.org_id.clone().into());
        record.data.insert("updated_at".into(), now.into());

        if record.deleted {
            db.prepare(&format!(
                "UPDATE {} SET deleted_at = ?1, updated_at = ?2 \
                 WHERE id = ?3 AND org_id = ?4",
                record.table
            ))
            .bind(&[
                JsValue::from_f64(now as f64),
                JsValue::from_f64(now as f64),
                JsValue::from_f64(record.id as f64),
                JsValue::from_str(&claims.org_id),
            ])?
            .run()
            .await?;
        } else {
            // Filter data keys against the allowed columns for this table
            let allowed = allowed_columns(&record.table);
            let cols: Vec<&str> = record
                .data
                .keys()
                .filter(|k| allowed.contains(&k.as_str()))
                .map(|k| k.as_str())
                .collect();

            if cols.is_empty() {
                continue;
            }

            let placeholders: Vec<String> =
                (1..=cols.len()).map(|i| format!("?{}", i)).collect();
            let update_clauses: Vec<String> = cols
                .iter()
                .map(|c| format!("{} = excluded.{}", c, c))
                .collect();

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({}) \
                 ON CONFLICT(id, org_id) DO UPDATE SET {}",
                record.table,
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            );

            let binds: Vec<JsValue> =
                cols.iter().map(|c| val_to_js(&record.data[*c])).collect();

            db.prepare(&sql).bind(&binds)?.run().await?;
        }

        synced += 1;
    }

    Response::from_json(&serde_json::json!({ "synced": synced, "server_time": now }))
}

pub async fn handle_list_users(req: Request, env: &Env) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    if claims.role != "admin" {
        return Response::error("Admin only", 403);
    }

    let db = env.d1("DB")?;
    let users = db
        .prepare(
            "SELECT id, email, role, is_active, invited_at, last_seen \
             FROM enterprise_users WHERE org_id = ?1 ORDER BY email",
        )
        .bind(&[JsValue::from_str(&claims.org_id)])?
        .all()
        .await?
        .results::<EnterpriseUser>()?;

    Response::from_json(&serde_json::json!({ "users": users }))
}

#[derive(Deserialize)]
struct InviteRequest {
    email: String,
    role: String,
}

pub async fn handle_invite(mut req: Request, env: &Env) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    if claims.role != "admin" {
        return Response::error("Admin only", 403);
    }

    let body: InviteRequest =
        req.json().await.map_err(|_| "Invalid request body")?;

    if !["admin", "editor", "viewer"].contains(&body.role.as_str()) {
        return Response::error("role must be admin, editor, or viewer", 400);
    }

    let db = env.d1("DB")?;

    let license = db
        .prepare(
            "SELECT seats, status, trial_ends_at FROM licenses WHERE org_id = ?1",
        )
        .bind(&[JsValue::from_str(&claims.org_id)])?
        .first::<serde_json::Value>(None)
        .await?
        .ok_or("No licence found")?;

    let now = jwt::now();
    let trial_ends = license
        .get("trial_ends_at")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let in_trial = license
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        == "trial"
        && trial_ends > now;
    let status = license
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if status != "active" && !in_trial {
        return Response::error("No active licence or trial", 402);
    }

    let seats = license
        .get("seats")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let active_count = db
        .prepare(
            "SELECT COUNT(*) as n FROM enterprise_users \
             WHERE org_id = ?1 AND is_active = 1",
        )
        .bind(&[JsValue::from_str(&claims.org_id)])?
        .first::<serde_json::Value>(None)
        .await?
        .and_then(|r| r.get("n").and_then(|v| v.as_i64()))
        .unwrap_or(0);

    if active_count >= seats {
        return Response::error("Seat limit reached — upgrade your plan", 402);
    }

    let id = uuid::Uuid::new_v4().to_string();
    db.prepare(
        "INSERT INTO enterprise_users (id, org_id, email, role, is_active, invited_at) \
         VALUES (?1, ?2, ?3, ?4, 0, ?5) \
         ON CONFLICT(email) DO UPDATE SET role = excluded.role, is_active = 1",
    )
    .bind(&[
        JsValue::from_str(&id),
        JsValue::from_str(&claims.org_id),
        JsValue::from_str(&body.email),
        JsValue::from_str(&body.role),
        JsValue::from_f64(now as f64),
    ])?
    .run()
    .await?;

    Ok(Response::from_json(
        &serde_json::json!({ "invited": body.email, "role": body.role }),
    )?
    .with_status(201))
}

pub async fn handle_change_role(
    mut req: Request,
    env: &Env,
    user_id: String,
) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    if claims.role != "admin" {
        return Response::error("Admin only", 403);
    }

    #[derive(Deserialize)]
    struct Body {
        role: String,
    }
    let body: Body = req.json().await.map_err(|_| "Invalid request body")?;
    if !["admin", "editor", "viewer"].contains(&body.role.as_str()) {
        return Response::error("Invalid role", 400);
    }

    let db = env.d1("DB")?;
    db.prepare(
        "UPDATE enterprise_users SET role = ?1, updated_at = ?2 \
         WHERE id = ?3 AND org_id = ?4",
    )
    .bind(&[
        JsValue::from_str(&body.role),
        JsValue::from_f64(jwt::now() as f64),
        JsValue::from_str(&user_id),
        JsValue::from_str(&claims.org_id),
    ])?
    .run()
    .await?;

    Response::from_json(&serde_json::json!({ "updated": user_id, "role": body.role }))
}

pub async fn handle_remove_user(
    req: Request,
    env: &Env,
    user_id: String,
) -> Result<Response> {
    let claims = authenticate(&req, env)?;
    if claims.role != "admin" {
        return Response::error("Admin only", 403);
    }

    let db = env.d1("DB")?;
    db.prepare(
        "UPDATE enterprise_users SET is_active = 0, updated_at = ?1 \
         WHERE id = ?2 AND org_id = ?3",
    )
    .bind(&[
        JsValue::from_f64(jwt::now() as f64),
        JsValue::from_str(&user_id),
        JsValue::from_str(&claims.org_id),
    ])?
    .run()
    .await?;

    Response::from_json(&serde_json::json!({ "removed": user_id }))
}

fn authenticate(req: &Request, env: &Env) -> Result<Claims> {
    let auth = req
        .headers()
        .get("Authorization")?
        .ok_or("Missing Authorization header")?;
    let token = auth.strip_prefix("Bearer ").unwrap_or(&auth).to_string();
    let secret = env.secret("JWT_SECRET")?.to_string();
    jwt::verify(&token, &secret).map_err(|e| e.to_string().into())
}

fn allowed_columns(table: &str) -> &'static [&'static str] {
    match table {
        "emission_sources" => &[
            "id", "org_id", "entity_id", "period_id", "scope",
            "scope2_method", "scope3_category", "category_name", "ghg_type",
            "activity_value", "activity_unit", "activity_source",
            "emission_factor_value", "emission_factor_unit",
            "emission_factor_source", "emission_factor_citation", "gwp_value",
            "emissions_tco2e", "biogenic_co2_tco2e", "uncertainty_pct",
            "is_excluded", "exclusion_reason", "notes", "deleted_at",
            "updated_at",
        ],
        "reporting_periods" => &[
            "id", "org_id", "year", "start_date", "end_date", "status",
            "gwp_ar_version", "deleted_at", "updated_at",
        ],
        "entities" => &[
            "id", "org_id", "name", "type", "ownership_pct",
            "is_financially_controlled", "is_operationally_controlled",
            "country_code", "is_active", "deleted_at", "updated_at",
        ],
        "ungc_cop" => &[
            "id", "org_id", "reporting_year", "status", "compliance_level",
            "ceo_statement_signed", "submitted_at", "deleted_at", "updated_at",
        ],
        "ungc_cop_responses" => &[
            "id", "org_id", "cop_id", "question_id", "question_text",
            "response", "auto_populated", "deleted_at", "updated_at",
        ],
        _ => &[],
    }
}

fn val_to_js(v: &serde_json::Value) -> JsValue {
    match v {
        serde_json::Value::String(s) => JsValue::from_str(s),
        serde_json::Value::Number(n) => {
            JsValue::from_f64(n.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::Bool(b) => JsValue::from_bool(*b),
        _ => JsValue::null(),
    }
}
