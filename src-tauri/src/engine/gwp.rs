// IPCC Global Warming Potential lookup
// Sources: AR4 (2007), AR5 (2013), AR6 (2021)
// 100-year time horizon (GWP-100) per ISO 14064-1:2018 requirement

use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ArVersion {
    AR4,
    AR5,
    AR6,
}

impl ArVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AR4 => "AR4",
            Self::AR5 => "AR5",
            Self::AR6 => "AR6",
        }
    }
}

impl std::fmt::Display for ArVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for ArVersion {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "AR4" => Ok(Self::AR4),
            "AR5" => Ok(Self::AR5),
            "AR6" => Ok(Self::AR6),
            other => Err(format!("Unknown AR version: {}", other)),
        }
    }
}

/// Fetch GWP value for a gas from the database.
/// Returns the GWP-100 value for CO₂e conversion.
/// Falls back to 1.0 (CO2) if the gas is not found — caller should handle missing gases.
pub fn get_gwp(conn: &Connection, gas: &str, ar_version: &ArVersion) -> Result<f64> {
    let result: Result<f64> = conn.query_row(
        "SELECT gwp_100 FROM gwp_values WHERE gas = ?1 AND ar_version = ?2",
        params![gas, ar_version.as_str()],
        |row| row.get(0),
    );

    match result {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // CO2 is always 1; for unknown gases return 0 to flag the issue
            if gas == "CO2" { Ok(1.0) } else { Ok(0.0) }
        }
        Err(e) => Err(e),
    }
}

/// Get all GWP values for a given AR version, returned as a map.
pub fn get_all_gwp(
    conn: &Connection,
    ar_version: &ArVersion,
) -> Result<std::collections::HashMap<String, f64>> {
    let mut stmt = conn.prepare(
        "SELECT gas, gwp_100 FROM gwp_values WHERE ar_version = ?1"
    )?;
    let map = stmt.query_map(params![ar_version.as_str()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?
    .collect::<Result<std::collections::HashMap<_, _>>>()?;
    Ok(map)
}
