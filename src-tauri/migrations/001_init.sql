-- c12 Carbon Accounting — Initial Schema
-- Standards: GRI 305, ISO 14064-1/2/3, UNGC COP

CREATE TABLE IF NOT EXISTS _migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at INTEGER NOT NULL
);

-- ─── Organization & Reporting ────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS organizations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  -- ISO 14064-1 §5.2: Organizational boundary method (exactly one required)
  boundary_method TEXT NOT NULL CHECK(boundary_method IN ('equity_share','financial_control','operational_control')),
  base_year INTEGER,
  reporting_currency TEXT NOT NULL DEFAULT 'USD',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Entities within the org boundary (subsidiaries, facilities, JVs)
CREATE TABLE IF NOT EXISTS entities (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  org_id INTEGER NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL CHECK(type IN ('parent','subsidiary','facility','jv','branch')),
  -- For equity share: ownership percentage (0–100)
  ownership_pct REAL,
  -- For financial/operational control: boolean flags
  is_financially_controlled INTEGER NOT NULL DEFAULT 0 CHECK(is_financially_controlled IN (0,1)),
  is_operationally_controlled INTEGER NOT NULL DEFAULT 0 CHECK(is_operationally_controlled IN (0,1)),
  country_code TEXT,
  -- GRI sector standard: 11=Oil&Gas, 12=Coal, 13=Agriculture, 14=Mining
  sector_gri INTEGER,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS reporting_periods (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  org_id INTEGER NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  year INTEGER NOT NULL,
  start_date TEXT NOT NULL,  -- ISO 8601
  end_date TEXT NOT NULL,    -- ISO 8601
  status TEXT NOT NULL DEFAULT 'draft'
    CHECK(status IN ('draft','in_review','verified','published')),
  -- IPCC AR version for GWP lookup; AR6 is current default
  gwp_ar_version TEXT NOT NULL DEFAULT 'AR6' CHECK(gwp_ar_version IN ('AR4','AR5','AR6')),
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE(org_id, year)
);

-- ─── Emission Sources ─────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS emission_sources (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  entity_id INTEGER NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,

  -- GHG Protocol / ISO 14064-1: Scope classification
  scope INTEGER NOT NULL CHECK(scope IN (1,2,3)),

  -- Scope 2 requires BOTH location_based AND market_based rows
  -- scope2_method is NULL for Scope 1 and 3
  scope2_method TEXT CHECK(scope2_method IN ('location_based','market_based') OR scope2_method IS NULL),

  -- GHG Protocol Corporate Value Chain Standard: 15 Scope 3 categories
  scope3_category INTEGER CHECK(scope3_category BETWEEN 1 AND 15 OR scope3_category IS NULL),

  -- Human-readable category: e.g. "Natural Gas Combustion", "Business Travel"
  category_name TEXT NOT NULL,

  -- GHG type (ISO 14064-1 requires accounting for all applicable gases)
  ghg_type TEXT NOT NULL CHECK(ghg_type IN (
    'CO2','CH4_non_fossil','CH4_fossil','N2O',
    'HFC','PFC','SF6','NF3','other'
  )),

  -- Activity data (the measured input)
  activity_value REAL NOT NULL CHECK(activity_value >= 0),
  activity_unit TEXT NOT NULL,   -- e.g. 'L', 'm3', 'kg', 'kWh', 'km', 't'
  activity_source TEXT,          -- Invoice | Meter | Supplier Report | Estimate

  -- Emission factor with full traceability (ISO 14064-1 §7.3)
  emission_factor_value REAL NOT NULL CHECK(emission_factor_value >= 0),
  emission_factor_unit TEXT NOT NULL,   -- e.g. 'kgCO2e/L', 'kgCO2e/kWh'
  emission_factor_source TEXT NOT NULL, -- 'IPCC AR6', 'GHG Protocol', 'DEFRA 2024', etc.
  emission_factor_citation TEXT,        -- URL or publication reference

  -- GWP value used (from gwp_values table)
  gwp_value REAL NOT NULL DEFAULT 1.0,

  -- Calculated result (stored immutably once calculated; NULL until calculated)
  emissions_tco2e REAL,

  -- GRI 305-1: Biogenic CO2 reported separately, never summed into GHG totals
  biogenic_co2_tco2e REAL,

  -- ISO 14064-1 §7.4: Uncertainty assessment
  uncertainty_pct REAL,  -- percentage uncertainty (e.g. 5.0 for ±5%)

  -- If excluded from totals, reason is required
  is_excluded INTEGER NOT NULL DEFAULT 0 CHECK(is_excluded IN (0,1)),
  exclusion_reason TEXT,

  notes TEXT,
  created_by TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

  -- Constraint: Scope 3 entries must have a category
  CHECK(scope != 3 OR scope3_category IS NOT NULL),
  -- Constraint: Scope 2 entries must have a method
  CHECK(scope != 2 OR scope2_method IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_sources_period ON emission_sources(period_id);
CREATE INDEX IF NOT EXISTS idx_sources_entity ON emission_sources(entity_id);
CREATE INDEX IF NOT EXISTS idx_sources_scope ON emission_sources(scope);

-- ─── Scope 2 Market-Based Contractual Instruments ────────────────────────────
-- GRI 305-2: Market-based calculation requires contractual instruments

CREATE TABLE IF NOT EXISTS contractual_instruments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id INTEGER NOT NULL REFERENCES emission_sources(id) ON DELETE CASCADE,
  instrument_type TEXT NOT NULL CHECK(instrument_type IN ('REC','PPA','GG','direct_agreement','other')),
  quantity_mwh REAL,
  vintage_year INTEGER,
  registry TEXT,
  -- Emission factor from the contractual instrument (tCO2e/MWh)
  supplier_emission_factor REAL,
  documentation TEXT
);

-- ─── Evidence Attachments ─────────────────────────────────────────────────────
-- ISO 14064-1 §8: Documentation and records

CREATE TABLE IF NOT EXISTS evidence (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id INTEGER NOT NULL REFERENCES emission_sources(id) ON DELETE CASCADE,
  filename TEXT NOT NULL,
  -- Relative path within app data directory
  file_path TEXT NOT NULL,
  file_type TEXT,
  file_size_bytes INTEGER,
  uploaded_at INTEGER NOT NULL DEFAULT (unixepoch()),
  uploaded_by TEXT
);

-- ─── GRI 305-4: Emissions Intensity ─────────────────────────────────────────

CREATE TABLE IF NOT EXISTS intensity_ratios (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,
  -- Which scopes are included in the numerator
  includes_scope1 INTEGER NOT NULL DEFAULT 1,
  includes_scope2 INTEGER NOT NULL DEFAULT 1,
  includes_scope3 INTEGER NOT NULL DEFAULT 0,
  total_emissions_tco2e REAL NOT NULL,
  -- Activity metric (denominator)
  metric_name TEXT NOT NULL,    -- e.g. 'Revenue (USD)', 'Units Produced', 'Employees', 'm² floor area'
  metric_value REAL NOT NULL,
  metric_unit TEXT NOT NULL,
  -- Calculated ratio
  intensity_ratio REAL NOT NULL,  -- tCO2e per unit of metric
  -- GRI 305-4: Scope 3 intensity reported separately if calculated
  scope3_intensity_ratio REAL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── GRI 305-5: Emissions Reductions ─────────────────────────────────────────
-- Excludes reductions from outsourcing or production cuts

CREATE TABLE IF NOT EXISTS reductions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,
  baseline_year INTEGER NOT NULL,
  baseline_tco2e REAL NOT NULL,
  current_tco2e REAL NOT NULL,
  reduction_tco2e REAL NOT NULL,
  reduction_pct REAL NOT NULL,
  methodology TEXT NOT NULL,
  -- GRI 305-5: These exclusions are mandatory — reductions from these sources must NOT be counted
  excludes_outsourcing INTEGER NOT NULL DEFAULT 1 CHECK(excludes_outsourcing = 1),
  excludes_production_cuts INTEGER NOT NULL DEFAULT 1 CHECK(excludes_production_cuts = 1),
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── GRI 305-6: ODS Emissions ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS ods_emissions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,
  substance TEXT NOT NULL,             -- e.g. 'R-22', 'R-410A', 'Halon-1301'
  production_metric_tons REAL NOT NULL DEFAULT 0,
  imports_metric_tons REAL NOT NULL DEFAULT 0,
  exports_metric_tons REAL NOT NULL DEFAULT 0,
  cfc11_equivalent REAL NOT NULL,      -- ODP-weighted total in CFC-11 equivalent
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── GRI 305-7: Supplemental Air Emissions ───────────────────────────────────

CREATE TABLE IF NOT EXISTS air_emissions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,
  emission_type TEXT NOT NULL CHECK(emission_type IN ('NOx','SOx','VOC','PM','HAP','other')),
  substance TEXT,
  value_metric_tons REAL NOT NULL CHECK(value_metric_tons >= 0),
  measurement_method TEXT NOT NULL CHECK(measurement_method IN ('direct_measurement','estimation','balance')),
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── ISO 14064-3: Verification Records ───────────────────────────────────────

CREATE TABLE IF NOT EXISTS verifications (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  period_id INTEGER NOT NULL REFERENCES reporting_periods(id) ON DELETE CASCADE,
  verifier_org TEXT,
  verifier_name TEXT,
  -- ISO 14065 accreditation details
  accreditation_body TEXT,       -- e.g. 'UKAS', 'ANAB', 'JAS-ANZ'
  accreditation_number TEXT,
  accreditation_expiry TEXT,
  -- Scope of verification
  scope_covered TEXT NOT NULL CHECK(scope_covered IN ('scope1','scope1_2','full')),
  -- ISO 14064-3 verification opinion
  opinion TEXT CHECK(opinion IN ('positive','qualified','adverse','no_opinion')),
  materiality_threshold_pct REAL, -- e.g. 5.0 for 5% materiality
  verification_date TEXT,
  report_path TEXT,              -- path to stored verification report file
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── GWP Reference Table ──────────────────────────────────────────────────────
-- Seeded in migration 002; values from IPCC AR4, AR5, AR6

CREATE TABLE IF NOT EXISTS gwp_values (
  gas TEXT NOT NULL,
  ar_version TEXT NOT NULL CHECK(ar_version IN ('AR4','AR5','AR6')),
  gwp_100 REAL NOT NULL,
  notes TEXT,
  PRIMARY KEY (gas, ar_version)
);

-- ─── Emission Factor Library ──────────────────────────────────────────────────
-- Seeded with IPCC, GHG Protocol, DEFRA UK factors; user-extendable

CREATE TABLE IF NOT EXISTS emission_factors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  category TEXT NOT NULL,    -- electricity, transport, fuel, refrigerant, waste, etc.
  subcategory TEXT,
  region TEXT,               -- 'GB', 'US', 'EU', 'AU', 'global'
  ghg_type TEXT NOT NULL,
  factor_value REAL NOT NULL,
  factor_unit TEXT NOT NULL, -- e.g. 'kgCO2e/kWh', 'kgCO2e/L', 'kgCO2e/km'
  ar_version TEXT,
  source TEXT NOT NULL,      -- 'IPCC AR6', 'GHG Protocol', 'DEFRA 2024', etc.
  source_url TEXT,
  valid_from TEXT,           -- year string, e.g. '2024'
  valid_until TEXT,
  -- 0 = seeded, 1 = user-created custom factor
  is_custom INTEGER NOT NULL DEFAULT 0 CHECK(is_custom IN (0,1)),
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── UNGC Communication on Progress ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS ungc_cop (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  org_id INTEGER NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  reporting_year INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft'
    CHECK(status IN ('draft','in_progress','complete','submitted')),
  compliance_level TEXT CHECK(compliance_level IN ('beginner','active','advanced','lead')),
  -- CEO Statement of Continued Support is mandatory before submission
  ceo_statement_signed INTEGER NOT NULL DEFAULT 0 CHECK(ceo_statement_signed IN (0,1)),
  ceo_name TEXT,
  signed_at INTEGER,
  submitted_at INTEGER,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE(org_id, reporting_year)
);

CREATE TABLE IF NOT EXISTS ungc_cop_responses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  cop_id INTEGER NOT NULL REFERENCES ungc_cop(id) ON DELETE CASCADE,
  -- Official 2025 UNGC questionnaire question IDs
  question_id TEXT NOT NULL,
  question_text TEXT NOT NULL,
  principle_reference TEXT,   -- e.g. 'P7', 'P8', 'P9'
  response TEXT,
  -- 1 if automatically populated from GRI 305 / ISO 14064 data
  auto_populated INTEGER NOT NULL DEFAULT 0,
  auto_source_table TEXT,
  auto_source_id INTEGER,
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE(cop_id, question_id)
);

-- ─── Enterprise: Users & Sync ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,   -- Cloudflare user ID (UUID)
  email TEXT UNIQUE NOT NULL,
  display_name TEXT,
  role TEXT NOT NULL CHECK(role IN ('admin','editor','viewer')),
  is_sso INTEGER NOT NULL DEFAULT 0,
  last_synced_at INTEGER,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS sync_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ─── Immutable Audit Log ──────────────────────────────────────────────────────
-- ISO 14064-1 §8.2: Permanent record of all data changes

CREATE TABLE IF NOT EXISTS audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  table_name TEXT NOT NULL,
  record_id INTEGER NOT NULL,
  action TEXT NOT NULL CHECK(action IN ('INSERT','UPDATE','DELETE')),
  field_name TEXT,
  old_value TEXT,
  new_value TEXT,
  user_id TEXT,
  timestamp INTEGER NOT NULL DEFAULT (unixepoch()),
  reason TEXT
) STRICT;

-- Audit log is append-only; no UPDATE or DELETE permitted (enforced in Rust)
CREATE INDEX IF NOT EXISTS idx_audit_table_record ON audit_log(table_name, record_id);
