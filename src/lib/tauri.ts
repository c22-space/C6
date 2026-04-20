import { invoke } from '@tauri-apps/api/core'

// Typed wrappers for Tauri commands

export interface Organization {
  id: number
  name: string
  boundary_method: 'equity_share' | 'financial_control' | 'operational_control'
  base_year: number | null
  reporting_currency: string
}

export interface Entity {
  id: number
  org_id: number
  name: string
  type: 'parent' | 'subsidiary' | 'facility' | 'jv' | 'branch'
  ownership_pct: number | null
  is_financially_controlled: boolean
  is_operationally_controlled: boolean
  country_code: string | null
  sector_gri: number | null
  is_active: boolean
}

export interface ReportingPeriod {
  id: number
  org_id: number
  year: number
  start_date: string
  end_date: string
  status: 'draft' | 'in_review' | 'verified' | 'published'
  gwp_ar_version: 'AR4' | 'AR5' | 'AR6'
}

export interface EmissionSource {
  id: number
  entity_id: number
  period_id: number
  scope: 1 | 2 | 3
  scope2_method: 'location_based' | 'market_based' | null
  scope3_category: number | null
  category_name: string
  ghg_type: string
  activity_value: number
  activity_unit: string
  activity_source: string | null
  emission_factor_value: number
  emission_factor_unit: string
  emission_factor_source: string
  emission_factor_citation: string | null
  gwp_value: number
  emissions_tco2e: number | null
  biogenic_co2_tco2e: number | null
  uncertainty_pct: number | null
  is_excluded: boolean
  exclusion_reason: string | null
  notes: string | null
}

export interface PeriodInventory {
  period_id: number
  year: number
  gwp_ar_version: string
  scope1: {
    gross_tco2e: number
    biogenic_co2_tco2e: number
    by_gas: Record<string, number>
    sources: EmissionSource[]
    combined_uncertainty_pct: number
  }
  scope2: {
    location_based_tco2e: number
    market_based_tco2e: number
    contractual_coverage_pct: number
    location_sources: EmissionSource[]
    market_sources: EmissionSource[]
  }
  scope3: {
    gross_tco2e: number
    upstream_tco2e: number
    downstream_tco2e: number
    categories: Array<{
      category: number
      category_name: string
      direction: string
      total_tco2e: number
      source_count: number
      is_excluded: boolean
      exclusion_reason: string | null
    }>
    excluded_categories: number[]
  }
  total_tco2e: number
  scope1_scope2_tco2e: number
}

// ── Org commands ─────────────────────────────────────────────────────────────

export const createOrg = (args: {
  name: string
  boundary_method: string
  base_year?: number | null
  reporting_currency?: string
}): Promise<Organization> => invoke('create_org', args)

export const listOrgs = (): Promise<Organization[]> => invoke('list_orgs')

export const getOrg = (id: number): Promise<Organization | null> => invoke('get_org', { id })

export const updateOrg = (args: {
  id: number
  name: string
  boundary_method: string
  base_year?: number | null
}): Promise<void> => invoke('update_org', args)

export const createEntity = (args: {
  org_id: number
  name: string
  type: string
  ownership_pct?: number | null
  is_financially_controlled: boolean
  is_operationally_controlled: boolean
  country_code?: string | null
  sector_gri?: number | null
}): Promise<Entity> => invoke('create_entity', args)

export const listEntities = (org_id: number): Promise<Entity[]> =>
  invoke('list_entities', { org_id })

export const createPeriod = (args: {
  org_id: number
  year: number
  start_date: string
  end_date: string
  gwp_ar_version?: string
}): Promise<ReportingPeriod> => invoke('create_period', args)

export const listPeriods = (org_id: number): Promise<ReportingPeriod[]> =>
  invoke('list_periods', { org_id })

// ── Source commands ───────────────────────────────────────────────────────────

export const createSource = (input: Omit<EmissionSource, 'id' | 'emissions_tco2e' | 'is_excluded'>): Promise<EmissionSource> =>
  invoke('create_source', { input })

export const updateSource = (id: number, input: Omit<EmissionSource, 'id' | 'emissions_tco2e' | 'is_excluded'>, reason?: string): Promise<EmissionSource> =>
  invoke('update_source', { id, input, reason })

export const deleteSource = (id: number, reason?: string): Promise<void> =>
  invoke('delete_source', { id, reason })

export const listSources = (period_id: number, scope?: number): Promise<EmissionSource[]> =>
  invoke('list_sources', { period_id, scope })

export const listEmissionFactors = (category?: string, region?: string) =>
  invoke<Array<Record<string, unknown>>>('list_emission_factors', { category, region })

// ── Calculation commands ──────────────────────────────────────────────────────

export const calculatePeriod = (period_id: number): Promise<PeriodInventory> =>
  invoke('calculate_period', { period_id })

export const listGwpValues = (ar_version: string) =>
  invoke<Array<Record<string, unknown>>>('list_gwp_values', { ar_version })

export const getAuditLog = (table_name: string, record_id: number) =>
  invoke<Array<Record<string, unknown>>>('get_audit_log', { table_name, record_id })

// ── Report commands ───────────────────────────────────────────────────────────

export const generateGri305Report = (period_id: number) =>
  invoke<Record<string, unknown>>('generate_gri305_report', { period_id })

export const exportSourcesCsv = (period_id: number, path: string) =>
  invoke<void>('export_sources_csv', { period_id, path })

// ── Intensity commands (GRI 305-4) ────────────────────────────────────────────

export interface IntensityResult {
  includes_scope1: boolean
  includes_scope2: boolean
  includes_scope3: boolean
  total_emissions_tco2e: number
  metric_name: string
  metric_value: number
  metric_unit: string
  intensity_ratio: number
  scope3_intensity_ratio: number | null
}

export const saveIntensityMetric = (args: {
  period_id: number
  metric_name: string
  metric_value: number
  metric_unit: string
  includes_scope1: boolean
  includes_scope2: boolean
  includes_scope3: boolean
}): Promise<IntensityResult> => invoke('save_intensity_metric', args)

export const listIntensityResults = (period_id: number): Promise<IntensityResult[]> =>
  invoke('list_intensity_results', { period_id })

export const deleteIntensityResult = (period_id: number, metric_name: string): Promise<void> =>
  invoke('delete_intensity_result', { period_id, metric_name })

// ── Reduction commands (GRI 305-5) ────────────────────────────────────────────

export interface Reduction {
  id: number
  period_id: number
  baseline_year: number
  baseline_tco2e: number
  current_tco2e: number
  reduction_tco2e: number
  reduction_pct: number
  methodology: string
}

export const listReductions = (period_id: number): Promise<Reduction[]> =>
  invoke('list_reductions', { period_id })

export const createReduction = (args: {
  period_id: number
  baseline_year: number
  baseline_tco2e: number
  current_tco2e: number
  methodology: string
}): Promise<Reduction> => invoke('create_reduction', args)

export const deleteReduction = (id: number): Promise<void> =>
  invoke('delete_reduction', { id })

// ── ODS commands (GRI 305-6) ──────────────────────────────────────────────────

export interface OdsEntry {
  id: number
  period_id: number
  substance: string
  production_metric_tons: number
  imports_metric_tons: number
  exports_metric_tons: number
  cfc11_equivalent: number
}

export const listOdsEmissions = (period_id: number): Promise<OdsEntry[]> =>
  invoke('list_ods_emissions', { period_id })

export const createOdsEmission = (args: {
  period_id: number
  substance: string
  production_metric_tons: number
  imports_metric_tons: number
  exports_metric_tons: number
  cfc11_equivalent: number
}): Promise<OdsEntry> => invoke('create_ods_emission', args)

export const deleteOdsEmission = (id: number): Promise<void> =>
  invoke('delete_ods_emission', { id })

// ── Air emission commands (GRI 305-7) ─────────────────────────────────────────

export interface AirEntry {
  id: number
  period_id: number
  emission_type: string
  substance: string | null
  value_metric_tons: number
  measurement_method: string
}

export const listAirEmissions = (period_id: number): Promise<AirEntry[]> =>
  invoke('list_air_emissions', { period_id })

export const createAirEmission = (args: {
  period_id: number
  emission_type: string
  substance?: string | null
  value_metric_tons: number
  measurement_method: string
}): Promise<AirEntry> => invoke('create_air_emission', args)

export const deleteAirEmission = (id: number): Promise<void> =>
  invoke('delete_air_emission', { id })

// ── UNGC commands ─────────────────────────────────────────────────────────────

export const initCop = (org_id: number, reporting_year: number) =>
  invoke<Record<string, unknown>>('init_cop', { org_id, reporting_year })

export const autoPopulateCop = (cop_id: number, period_id: number) =>
  invoke<number>('auto_populate_cop', { cop_id, period_id })

export const getCopQuestions = (cop_id: number) =>
  invoke<Array<Record<string, unknown>>>('get_cop_questions', { cop_id })

export const saveCopResponse = (cop_id: number, question_id: string, response: string) =>
  invoke<void>('save_cop_response', { cop_id, question_id, response })

export const signCeoStatement = (cop_id: number, ceo_name: string) =>
  invoke<void>('sign_ceo_statement', { cop_id, ceo_name })

export const computeComplianceLevel = (cop_id: number) =>
  invoke<string>('compute_compliance_level', { cop_id })
