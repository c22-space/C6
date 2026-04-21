use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::{Env, Request, Response, Result};

#[derive(Deserialize)]
struct UpdateManifest {
    version: String,
    notes: String,
    pub_date: String,
    platforms: HashMap<String, PlatformEntry>,
}

#[derive(Deserialize, Serialize)]
struct PlatformEntry {
    signature: String,
    url: String,
}

pub async fn handle(req: Request, env: &Env) -> Result<Response> {
    let url = req.url()?;
    let mut platform = None;
    let mut client_version = None;

    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "platform" => platform = Some(v.into_owned()),
            "version" => client_version = Some(v.into_owned()),
            _ => {}
        }
    }

    let platform = platform.ok_or("Missing platform")?;
    let client_version = client_version.ok_or("Missing version")?;

    let bucket = env.bucket("UPDATES")?;
    let obj = bucket.get("latest.json").execute().await?;

    let obj = match obj {
        Some(o) => o,
        None => return Response::error("No updates available", 204),
    };

    let text = obj.body().ok_or("Object has no body")?.text().await?;
    let manifest: UpdateManifest =
        serde_json::from_str(&text).map_err(|_| "Malformed manifest")?;

    if !is_newer(&manifest.version, &client_version) {
        return Response::empty()
            .map(|r| r.with_status(204));
    }

    let platform_data = manifest
        .platforms
        .get(&platform)
        .ok_or("Platform not supported")?;

    Response::from_json(&serde_json::json!({
        "version":   manifest.version,
        "notes":     manifest.notes,
        "pub_date":  manifest.pub_date,
        "url":       platform_data.url,
        "signature": platform_data.signature,
    }))
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let s = v.trim_start_matches('v');
        let mut parts = s.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    };
    let (la, lb, lc) = parse(latest);
    let (ca, cb, cc) = parse(current);
    (la, lb, lc) > (ca, cb, cc)
}
