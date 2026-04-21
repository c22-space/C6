use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const ACCESS_TTL: i64 = 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub org_id: String,
    pub role: String,
    pub plan: String,
    pub trial_ends_at: Option<i64>,
    pub iat: i64,
    pub exp: i64,
}

pub fn now() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}

pub fn sign(claims: &Claims, secret: &str) -> Result<String, String> {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let body = URL_SAFE_NO_PAD
        .encode(serde_json::to_string(claims).map_err(|e| e.to_string())?);
    let input = format!("{}.{}", header, body);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| e.to_string())?;
    mac.update(input.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{}.{}", input, sig))
}

pub fn verify(token: &str, secret: &str) -> Result<Claims, String> {
    let mut parts = token.splitn(3, '.');
    let header = parts.next().ok_or("Malformed JWT")?;
    let payload = parts.next().ok_or("Malformed JWT")?;
    let sig_b64 = parts.next().ok_or("Malformed JWT")?;

    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| "Invalid signature encoding")?;
    let input = format!("{}.{}", header, payload);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| e.to_string())?;
    mac.update(input.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| "Invalid signature".to_string())?;

    let payload_bytes =
        URL_SAFE_NO_PAD.decode(payload).map_err(|e| e.to_string())?;
    let claims: Claims =
        serde_json::from_slice(&payload_bytes).map_err(|e| e.to_string())?;

    if claims.exp < now() {
        return Err("Token expired".into());
    }
    Ok(claims)
}

pub fn decode_cf_payload(token: &str) -> Result<serde_json::Value, String> {
    let payload = token.split('.').nth(1).ok_or("Malformed JWT")?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
