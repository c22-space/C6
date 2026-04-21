-- D1 schema for c12 Enterprise (Cloudflare D1)
-- Applied via: wrangler d1 migrations apply c12-enterprise

-- Enterprise licence tracking (synced from DodoPayments webhooks)
CREATE TABLE IF NOT EXISTS licenses (
  org_id TEXT PRIMARY KEY,
  dodo_subscription_id TEXT UNIQUE NOT NULL,
  seats INTEGER NOT NULL DEFAULT 5,
  tier TEXT NOT NULL DEFAULT 'enterprise',
  status TEXT NOT NULL CHECK(status IN ('trial','active','past_due','cancelled')),
  trial_ends_at INTEGER,            -- unix timestamp; NULL for paid subscriptions
  expires_at TEXT,                  -- ISO date; NULL if subscription is ongoing
  updated_at INTEGER NOT NULL
);

-- Enterprise users (provisioned via SSO + invite)
CREATE TABLE IF NOT EXISTS enterprise_users (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES licenses(org_id),
  email TEXT UNIQUE NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('admin','editor','viewer')),
  is_active INTEGER NOT NULL DEFAULT 0,
  invited_at INTEGER NOT NULL,
  last_seen INTEGER,
  updated_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_eu_org ON enterprise_users(org_id);

-- Synced emission sources (delta sync from local SQLite)
CREATE TABLE IF NOT EXISTS emission_sources (
  id INTEGER NOT NULL,
  org_id TEXT NOT NULL,
  entity_id INTEGER,
  period_id INTEGER NOT NULL,
  scope INTEGER NOT NULL,
  scope2_method TEXT,
  scope3_category INTEGER,
  category_name TEXT NOT NULL,
  ghg_type TEXT NOT NULL,
  activity_value REAL NOT NULL,
  activity_unit TEXT NOT NULL,
  activity_source TEXT,
  emission_factor_value REAL NOT NULL,
  emission_factor_unit TEXT NOT NULL,
  emission_factor_source TEXT NOT NULL,
  emission_factor_citation TEXT,
  gwp_value REAL NOT NULL,
  emissions_tco2e REAL,
  biogenic_co2_tco2e REAL,
  uncertainty_pct REAL,
  is_excluded INTEGER DEFAULT 0,
  exclusion_reason TEXT,
  notes TEXT,
  deleted_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (id, org_id)
);

CREATE INDEX IF NOT EXISTS idx_es_org_period ON emission_sources(org_id, period_id);

-- Synced reporting periods
CREATE TABLE IF NOT EXISTS reporting_periods (
  id INTEGER NOT NULL,
  org_id TEXT NOT NULL,
  year INTEGER NOT NULL,
  start_date TEXT NOT NULL,
  end_date TEXT NOT NULL,
  status TEXT DEFAULT 'draft',
  gwp_ar_version TEXT DEFAULT 'AR6',
  deleted_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (id, org_id)
);

-- Synced entities
CREATE TABLE IF NOT EXISTS entities (
  id INTEGER NOT NULL,
  org_id TEXT NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  ownership_pct REAL,
  is_financially_controlled INTEGER,
  is_operationally_controlled INTEGER,
  country_code TEXT,
  is_active INTEGER DEFAULT 1,
  deleted_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (id, org_id)
);

-- Synced UNGC COP
CREATE TABLE IF NOT EXISTS ungc_cop (
  id INTEGER NOT NULL,
  org_id TEXT NOT NULL,
  reporting_year INTEGER NOT NULL,
  status TEXT DEFAULT 'draft',
  compliance_level TEXT,
  ceo_statement_signed INTEGER DEFAULT 0,
  submitted_at INTEGER,
  deleted_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (id, org_id)
);

CREATE TABLE IF NOT EXISTS ungc_cop_responses (
  id INTEGER NOT NULL,
  org_id TEXT NOT NULL,
  cop_id INTEGER NOT NULL,
  question_id TEXT NOT NULL,
  question_text TEXT NOT NULL,
  response TEXT,
  auto_populated INTEGER DEFAULT 0,
  deleted_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (id, org_id)
);
