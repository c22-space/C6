use serde::Deserialize;
use worker::{wasm_bindgen::JsValue, Env, Request, Response, Result};

use crate::jwt;

#[derive(Deserialize)]
struct DbUser {
    id: String,
    org_id: String,
    role: String,
}

#[derive(Deserialize)]
struct DbLicense {
    seats: i64,
    status: String,
    trial_ends_at: Option<i64>,
}

#[derive(Deserialize)]
struct CountRow {
    n: i64,
}

pub async fn handle_token(req: Request, env: &Env) -> Result<Response> {
    let cf_jwt = req
        .headers()
        .get("Cf-Access-Jwt-Assertion")?
        .ok_or("Missing Cloudflare Access JWT")?;

    let cf_claims =
        jwt::decode_cf_payload(&cf_jwt).map_err(|e| e.to_string())?;
    let email = cf_claims
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or("No email in Cloudflare Access JWT")?
        .to_string();

    let db = env.d1("DB")?;
    let secret = env.secret("JWT_SECRET")?.to_string();

    let user = db
        .prepare(
            "SELECT id, org_id, role FROM enterprise_users \
             WHERE email = ?1 AND is_active = 1",
        )
        .bind(&[JsValue::from_str(&email)])?
        .first::<DbUser>(None)
        .await?
        .ok_or("User not provisioned")?;

    let license = db
        .prepare(
            "SELECT seats, status, trial_ends_at FROM licenses WHERE org_id = ?1",
        )
        .bind(&[JsValue::from_str(&user.org_id)])?
        .first::<DbLicense>(None)
        .await?
        .ok_or("No active Enterprise licence or trial")?;

    let now = jwt::now();
    let in_trial = license.status == "trial"
        && license.trial_ends_at.map(|t| t > now).unwrap_or(false);

    if license.status != "active" && !in_trial {
        return Response::error("No active Enterprise licence or trial", 402);
    }

    let active_n = db
        .prepare(
            "SELECT COUNT(*) as n FROM enterprise_users \
             WHERE org_id = ?1 AND is_active = 1",
        )
        .bind(&[JsValue::from_str(&user.org_id)])?
        .first::<CountRow>(None)
        .await?
        .map(|r| r.n)
        .unwrap_or(0);

    if active_n > license.seats {
        return Response::error("Seat limit exceeded", 402);
    }

    let claims = jwt::Claims {
        sub: user.id,
        email,
        org_id: user.org_id,
        role: user.role,
        plan: if in_trial { "trial".into() } else { "enterprise".into() },
        trial_ends_at: if in_trial { license.trial_ends_at } else { None },
        iat: now,
        exp: now + jwt::ACCESS_TTL,
    };

    let token = jwt::sign(&claims, &secret).map_err(|e| e.to_string())?;
    Response::from_json(&serde_json::json!({ "token": token, "expires_in": jwt::ACCESS_TTL }))
}

pub async fn handle_refresh(req: Request, env: &Env) -> Result<Response> {
    let auth = req
        .headers()
        .get("Authorization")?
        .ok_or("Missing Authorization header")?;
    let token = auth.strip_prefix("Bearer ").unwrap_or(&auth).to_string();
    let secret = env.secret("JWT_SECRET")?.to_string();

    let claims =
        jwt::verify(&token, &secret).map_err(|_| "Invalid or expired token")?;
    let now = jwt::now();
    let new_claims = jwt::Claims { iat: now, exp: now + jwt::ACCESS_TTL, ..claims };
    let token = jwt::sign(&new_claims, &secret).map_err(|e| e.to_string())?;
    Response::from_json(
        &serde_json::json!({ "token": token, "expires_in": jwt::ACCESS_TTL }),
    )
}

#[derive(Deserialize)]
struct TrialRequest {
    org_id: String,
    org_name: String,
}

pub async fn handle_trial(mut req: Request, env: &Env) -> Result<Response> {
    let cf_jwt = req
        .headers()
        .get("Cf-Access-Jwt-Assertion")?
        .ok_or("Missing Cloudflare Access JWT")?;

    let cf_claims =
        jwt::decode_cf_payload(&cf_jwt).map_err(|e| e.to_string())?;
    let email = cf_claims
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or("No email in Cloudflare Access JWT")?
        .to_string();

    let body: TrialRequest = req
        .json()
        .await
        .map_err(|_| "Invalid request body")?;

    if body.org_id.is_empty() || body.org_name.is_empty() {
        return Response::error("org_id and org_name required", 400);
    }

    let db = env.d1("DB")?;

    let existing = db
        .prepare("SELECT status FROM licenses WHERE org_id = ?1")
        .bind(&[JsValue::from_str(&body.org_id)])?
        .first::<serde_json::Value>(None)
        .await?;

    if existing.is_some() {
        return Response::error(
            "Trial or subscription already exists for this organisation",
            409,
        );
    }

    let now = jwt::now();
    let trial_ends_at = now + 14 * 86400;

    let user_id = uuid::Uuid::new_v4().to_string();
    db.prepare(
        "INSERT INTO enterprise_users (id, org_id, email, role, is_active, invited_at) \
         VALUES (?1, ?2, ?3, 'admin', 1, ?4) ON CONFLICT(email) DO NOTHING",
    )
    .bind(&[
        JsValue::from_str(&user_id),
        JsValue::from_str(&body.org_id),
        JsValue::from_str(&email),
        JsValue::from_f64(now as f64),
    ])?
    .run()
    .await?;

    db.prepare(
        "INSERT INTO licenses \
         (org_id, dodo_subscription_id, seats, tier, status, trial_ends_at, expires_at, updated_at) \
         VALUES (?1, ?2, 5, 'enterprise', 'trial', ?3, NULL, ?4) \
         ON CONFLICT(org_id) DO NOTHING",
    )
    .bind(&[
        JsValue::from_str(&body.org_id),
        JsValue::from_str(&format!("trial-{}", body.org_id)),
        JsValue::from_f64(trial_ends_at as f64),
        JsValue::from_f64(now as f64),
    ])?
    .run()
    .await?;

    let trial_ends_date = format_date(trial_ends_at);
    let message =
        format!("14-day free trial started. Ends {}.", trial_ends_date);

    Ok(Response::from_json(&serde_json::json!({
        "trial": true,
        "seats": 5,
        "trial_ends_at": trial_ends_at,
        "trial_ends_date": trial_ends_date,
        "message": message,
    }))?
    .with_status(201))
}

fn format_date(ts: i64) -> String {
    let date =
        js_sys::Date::new(&JsValue::from_f64((ts * 1000) as f64));
    format!(
        "{}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}
