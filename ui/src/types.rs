use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Organization {
    pub id: i64,
    pub name: String,
    pub boundary_method: String,
    pub base_year: Option<i32>,
    pub reporting_currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: i64,
    pub org_id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub ownership_pct: Option<f64>,
    pub is_financially_controlled: bool,
    pub is_operationally_controlled: bool,
    pub country_code: Option<String>,
    pub sector_gri: Option<i32>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportingPeriod {
    pub id: i64,
    pub org_id: i64,
    pub year: i32,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub gwp_ar_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmissionSource {
    pub id: i64,
    pub entity_id: i64,
    pub period_id: i64,
    pub scope: i32,
    pub scope2_method: Option<String>,
    pub scope3_category: Option<i32>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope1Data {
    pub gross_tco2e: f64,
    pub biogenic_co2_tco2e: f64,
    pub by_gas: HashMap<String, f64>,
    pub sources: Vec<EmissionSource>,
    pub combined_uncertainty_pct: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope2Data {
    pub location_based_tco2e: f64,
    pub market_based_tco2e: f64,
    pub contractual_coverage_pct: f64,
    pub location_sources: Vec<EmissionSource>,
    pub market_sources: Vec<EmissionSource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope3Category {
    pub category: i32,
    pub category_name: String,
    pub direction: String,
    pub total_tco2e: f64,
    pub source_count: i32,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope3Data {
    pub gross_tco2e: f64,
    pub upstream_tco2e: f64,
    pub downstream_tco2e: f64,
    pub categories: Vec<Scope3Category>,
    pub excluded_categories: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeriodInventory {
    pub period_id: i64,
    pub year: i32,
    pub gwp_ar_version: String,
    pub scope1: Scope1Data,
    pub scope2: Scope2Data,
    pub scope3: Scope3Data,
    pub total_tco2e: f64,
    pub scope1_scope2_tco2e: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntensityResult {
    pub includes_scope1: bool,
    pub includes_scope2: bool,
    pub includes_scope3: bool,
    pub total_emissions_tco2e: f64,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: String,
    pub intensity_ratio: f64,
    pub scope3_intensity_ratio: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reduction {
    pub id: i64,
    pub period_id: i64,
    pub baseline_year: i32,
    pub baseline_tco2e: f64,
    pub current_tco2e: f64,
    pub reduction_tco2e: f64,
    pub reduction_pct: f64,
    pub methodology: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OdsEntry {
    pub id: i64,
    pub period_id: i64,
    pub substance: String,
    pub production_metric_tons: f64,
    pub imports_metric_tons: f64,
    pub exports_metric_tons: f64,
    pub cfc11_equivalent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AirEntry {
    pub id: i64,
    pub period_id: i64,
    pub emission_type: String,
    pub substance: Option<String>,
    pub value_metric_tons: f64,
    pub measurement_method: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CopQuestion {
    pub question_id: String,
    pub question_text: String,
    pub response: Option<String>,
    pub auto_populated: bool,
    pub source_table: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cop {
    pub id: i64,
    pub org_id: i64,
    pub reporting_year: i32,
    pub status: String,
    pub compliance_level: Option<String>,
    pub ceo_statement_signed: bool,
    pub submitted_at: Option<i64>,
}

pub type AuditEntry = HashMap<String, serde_json::Value>;
pub type ReportData = HashMap<String, serde_json::Value>;
