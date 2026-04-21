use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use worker::{wasm_bindgen::JsValue, Env, Request, Response, Result};

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct DodoEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: DodoData,
}

#[derive(Deserialize)]
struct DodoData {
    subscription_id: String,
    organization_id: Option<String>,
    seats: Option<i64>,
    status: Option<String>,
    tier: Option<String>,
    expires_at: Option<String>,
}

pub async fn handle(mut req: Request, env: &Env) -> Result<Response> {
    let signature = req
        .headers()
        .get("dodo-signature")?
        .ok_or("Missing signature")?;

    let body = req.text().await?;
    let secret = env.secret("DODO_WEBHOOK_SECRET")?.to_string();

    if !verify_sig(body.as_bytes(), &signature, &secret) {
        return Response::error("Invalid signature", 401);
    }

    let event: DodoEvent =
        serde_json::from_str(&body).map_err(|_| "Invalid JSON")?;

    let db = env.d1("DB")?;
    let now = crate::jwt::now();

    match event.event_type.as_str() {
        "subscription.created" | "subscription.updated" => {
            let org_id =
                event.data.organization_id.ok_or("Missing organization_id")?;
            let seats = event.data.seats.unwrap_or(5);
            let tier = event.data.tier.unwrap_or_else(|| "enterprise".into());
            let status =
                event.data.status.unwrap_or_else(|| "active".into());

            db.prepare(
                "INSERT INTO licenses \
                 (org_id, dodo_subscription_id, seats, tier, status, trial_ends_at, expires_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7) \
                 ON CONFLICT(org_id) DO UPDATE SET \
                   dodo_subscription_id = excluded.dodo_subscription_id, \
                   seats = excluded.seats, tier = excluded.tier, \
                   status = 'active', trial_ends_at = NULL, \
                   expires_at = excluded.expires_at, updated_at = excluded.updated_at",
            )
            .bind(&[
                JsValue::from_str(&org_id),
                JsValue::from_str(&event.data.subscription_id),
                JsValue::from_f64(seats as f64),
                JsValue::from_str(&tier),
                JsValue::from_str(&status),
                event
                    .data
                    .expires_at
                    .as_deref()
                    .map(JsValue::from_str)
                    .unwrap_or(JsValue::null()),
                JsValue::from_f64(now as f64),
            ])?
            .run()
            .await?;
        }
        "subscription.cancelled" | "subscription.expired" => {
            db.prepare(
                "UPDATE licenses SET status = 'cancelled', updated_at = ?1 \
                 WHERE dodo_subscription_id = ?2",
            )
            .bind(&[
                JsValue::from_f64(now as f64),
                JsValue::from_str(&event.data.subscription_id),
            ])?
            .run()
            .await?;
        }
        other => {
            worker::console_log!("Unhandled DodoPayments event: {}", other);
        }
    }

    Response::ok("OK")
}

fn verify_sig(body: &[u8], signature: &str, secret: &str) -> bool {
    let hex_part = match signature.strip_prefix("sha256=") {
        Some(h) => h,
        None => return false,
    };
    let expected = match hex::decode(hex_part) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}
