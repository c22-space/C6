-- c12 Seed Data: GWP values and emission factor library

-- ─── IPCC GWP Values ──────────────────────────────────────────────────────────
-- Source: IPCC Sixth Assessment Report (AR6), Chapter 7, Table 7.SM.7 (2021)
-- Source: IPCC Fifth Assessment Report (AR5), Table 8.A.1 (2013)
-- Source: IPCC Fourth Assessment Report (AR4), Table 2.14 (2007)

-- AR6 values (100-year GWP, biogenic methane treatment per IPCC 2021)
INSERT OR IGNORE INTO gwp_values (gas, ar_version, gwp_100, notes) VALUES
  ('CO2',          'AR6', 1.0,     NULL),
  ('CH4_non_fossil','AR6', 27.0,   'Biogenic/non-fossil methane, includes climate-carbon feedbacks'),
  ('CH4_fossil',   'AR6', 29.8,    'Fossil methane (coal mining, oil/gas systems), includes climate-carbon feedbacks'),
  ('N2O',          'AR6', 273.0,   NULL),
  ('SF6',          'AR6', 23500.0, NULL),
  ('NF3',          'AR6', 17400.0, NULL),
  ('HFC-23',       'AR6', 14600.0, NULL),
  ('HFC-32',       'AR6', 771.0,   NULL),
  ('HFC-41',       'AR6', 135.0,   NULL),
  ('HFC-125',      'AR6', 3740.0,  NULL),
  ('HFC-134',      'AR6', 1120.0,  NULL),
  ('HFC-134a',     'AR6', 1530.0,  NULL),
  ('HFC-143a',     'AR6', 5810.0,  NULL),
  ('HFC-152a',     'AR6', 164.0,   NULL),
  ('HFC-227ea',    'AR6', 3600.0,  NULL),
  ('HFC-236fa',    'AR6', 8690.0,  NULL),
  ('HFC-245fa',    'AR6', 962.0,   NULL),
  ('HFC-365mfc',   'AR6', 914.0,   NULL),
  ('HFC-43-10mee', 'AR6', 1650.0,  NULL),
  ('PFC-14',       'AR6', 7380.0,  'CF4'),
  ('PFC-116',      'AR6', 12400.0, 'C2F6'),
  ('PFC-218',      'AR6', 9290.0,  'C3F8'),
  ('PFC-31-10',    'AR6', 10000.0, 'C4F10'),
  ('PFC-41-12',    'AR6', 9220.0,  'C5F12'),
  ('PFC-51-14',    'AR6', 8620.0,  'C6F14');

-- AR5 values (100-year GWP, IPCC 2013)
INSERT OR IGNORE INTO gwp_values (gas, ar_version, gwp_100, notes) VALUES
  ('CO2',          'AR5', 1.0,    NULL),
  ('CH4_non_fossil','AR5', 28.0,  'Non-fossil methane, no climate-carbon feedbacks'),
  ('CH4_fossil',   'AR5', 30.0,   'Fossil methane, no climate-carbon feedbacks'),
  ('N2O',          'AR5', 265.0,  NULL),
  ('SF6',          'AR5', 23500.0,NULL),
  ('NF3',          'AR5', 16100.0,NULL),
  ('HFC-23',       'AR5', 12400.0,NULL),
  ('HFC-32',       'AR5', 677.0,  NULL),
  ('HFC-125',      'AR5', 3170.0, NULL),
  ('HFC-134a',     'AR5', 1300.0, NULL),
  ('HFC-143a',     'AR5', 4800.0, NULL),
  ('HFC-152a',     'AR5', 138.0,  NULL),
  ('HFC-227ea',    'AR5', 3350.0, NULL),
  ('HFC-236fa',    'AR5', 8060.0, NULL),
  ('HFC-245fa',    'AR5', 858.0,  NULL),
  ('PFC-14',       'AR5', 6630.0, 'CF4'),
  ('PFC-116',      'AR5', 11100.0,'C2F6'),
  ('PFC-218',      'AR5', 8900.0, 'C3F8');

-- AR4 values (100-year GWP, IPCC 2007)
INSERT OR IGNORE INTO gwp_values (gas, ar_version, gwp_100, notes) VALUES
  ('CO2',          'AR4', 1.0,    NULL),
  ('CH4_non_fossil','AR4', 25.0,  'No distinction between fossil/non-fossil in AR4'),
  ('CH4_fossil',   'AR4', 25.0,   NULL),
  ('N2O',          'AR4', 298.0,  NULL),
  ('SF6',          'AR4', 22800.0,NULL),
  ('NF3',          'AR4', 17200.0,NULL),
  ('HFC-23',       'AR4', 14800.0,NULL),
  ('HFC-32',       'AR4', 675.0,  NULL),
  ('HFC-125',      'AR4', 3500.0, NULL),
  ('HFC-134a',     'AR4', 1430.0, NULL),
  ('HFC-143a',     'AR4', 4470.0, NULL),
  ('HFC-152a',     'AR4', 124.0,  NULL),
  ('HFC-227ea',    'AR4', 3220.0, NULL),
  ('HFC-236fa',    'AR4', 9810.0, NULL),
  ('HFC-245fa',    'AR4', 1030.0, NULL),
  ('PFC-14',       'AR4', 7390.0, 'CF4'),
  ('PFC-116',      'AR4', 12200.0,'C2F6'),
  ('PFC-218',      'AR4', 8830.0, 'C3F8');

-- ─── Emission Factor Library (Starter Set) ───────────────────────────────────
-- Source: DEFRA/BEIS UK Greenhouse Gas Conversion Factor Repository 2024
-- Source: EPA eGRID 2022 (US)
-- Source: IEA Emission Factors 2023

-- Electricity — UK (DEFRA 2024, location-based, kgCO2e/kWh)
INSERT OR IGNORE INTO emission_factors
  (name, category, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('UK Grid Electricity (location-based)', 'electricity', 'GB', 'CO2', 0.20493, 'kgCO2e/kWh', 'AR5', 'DEFRA 2024', '2024'),
  ('US Average Grid (location-based)', 'electricity', 'US', 'CO2', 0.38600, 'kgCO2e/kWh', 'AR5', 'EPA eGRID 2022', '2022'),
  ('EU Average Grid (location-based)', 'electricity', 'EU', 'CO2', 0.27600, 'kgCO2e/kWh', 'AR5', 'IEA 2023', '2023'),
  ('Australia Grid (location-based)', 'electricity', 'AU', 'CO2', 0.61900, 'kgCO2e/kWh', 'AR5', 'Australian Government 2023', '2023');

-- Natural gas combustion (DEFRA 2024, kgCO2e/kWh gross CV)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Natural Gas (gross CV)', 'fuel', 'stationary_combustion', 'GB', 'CO2', 0.18270, 'kgCO2e/kWh', 'AR5', 'DEFRA 2024', '2024'),
  ('Natural Gas (kgCO2e/m3)', 'fuel', 'stationary_combustion', 'global', 'CO2', 1.93900, 'kgCO2e/m3', 'AR5', 'GHG Protocol', '2023');

-- Diesel & petrol (DEFRA 2024)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Diesel (road transport)', 'fuel', 'transport', 'GB', 'CO2', 2.51580, 'kgCO2e/L', 'AR5', 'DEFRA 2024', '2024'),
  ('Petrol/Gasoline (road transport)', 'fuel', 'transport', 'GB', 'CO2', 2.30300, 'kgCO2e/L', 'AR5', 'DEFRA 2024', '2024'),
  ('LPG (road transport)', 'fuel', 'transport', 'GB', 'CO2', 1.63400, 'kgCO2e/L', 'AR5', 'DEFRA 2024', '2024');

-- Aviation (DEFRA 2024, kgCO2e/passenger-km, with radiative forcing = 2x)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Air travel — domestic (with RF)', 'transport', 'business_travel', 'GB', 'CO2', 0.24500, 'kgCO2e/pkm', 'AR5', 'DEFRA 2024', '2024'),
  ('Air travel — short-haul (with RF)', 'transport', 'business_travel', 'GB', 'CO2', 0.15100, 'kgCO2e/pkm', 'AR5', 'DEFRA 2024', '2024'),
  ('Air travel — long-haul economy (with RF)', 'transport', 'business_travel', 'GB', 'CO2', 0.14890, 'kgCO2e/pkm', 'AR5', 'DEFRA 2024', '2024'),
  ('Air travel — long-haul business (with RF)', 'transport', 'business_travel', 'GB', 'CO2', 0.42900, 'kgCO2e/pkm', 'AR5', 'DEFRA 2024', '2024');

-- Road transport (DEFRA 2024, kgCO2e/km for average-sized car)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Car — average petrol (per km)', 'transport', 'business_travel', 'GB', 'CO2', 0.17004, 'kgCO2e/km', 'AR5', 'DEFRA 2024', '2024'),
  ('Car — average diesel (per km)', 'transport', 'business_travel', 'GB', 'CO2', 0.16284, 'kgCO2e/km', 'AR5', 'DEFRA 2024', '2024'),
  ('Car — electric (UK grid, per km)', 'transport', 'business_travel', 'GB', 'CO2', 0.05250, 'kgCO2e/km', 'AR5', 'DEFRA 2024', '2024'),
  ('Rail — national rail UK (per km)', 'transport', 'business_travel', 'GB', 'CO2', 0.03549, 'kgCO2e/pkm', 'AR5', 'DEFRA 2024', '2024');

-- Waste (DEFRA 2024, kgCO2e/tonne)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Mixed waste — landfill', 'waste', 'operations', 'GB', 'CO2', 467.00, 'kgCO2e/t', 'AR5', 'DEFRA 2024', '2024'),
  ('Mixed waste — recycling', 'waste', 'operations', 'GB', 'CO2', 21.30, 'kgCO2e/t', 'AR5', 'DEFRA 2024', '2024'),
  ('Mixed waste — combustion', 'waste', 'operations', 'GB', 'CO2', 443.20, 'kgCO2e/t', 'AR5', 'DEFRA 2024', '2024'),
  ('Paper/cardboard — landfill', 'waste', 'operations', 'GB', 'CO2', 565.60, 'kgCO2e/t', 'AR5', 'DEFRA 2024', '2024');

-- Refrigerants (DEFRA 2024, kgCO2e/kg)
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('R-22 (HCFC-22)', 'refrigerant', 'fugitive', 'global', 'HFC', 1760.0, 'kgCO2e/kg', 'AR5', 'DEFRA 2024', '2024'),
  ('R-134a (HFC-134a)', 'refrigerant', 'fugitive', 'global', 'HFC', 1430.0, 'kgCO2e/kg', 'AR4', 'DEFRA 2024', '2024'),
  ('R-404A', 'refrigerant', 'fugitive', 'global', 'HFC', 3920.0, 'kgCO2e/kg', 'AR4', 'DEFRA 2024', '2024'),
  ('R-410A', 'refrigerant', 'fugitive', 'global', 'HFC', 2090.0, 'kgCO2e/kg', 'AR4', 'DEFRA 2024', '2024'),
  ('R-32 (HFC-32)', 'refrigerant', 'fugitive', 'global', 'HFC', 675.0, 'kgCO2e/kg', 'AR4', 'DEFRA 2024', '2024'),
  ('SF6 (electrical equipment)', 'refrigerant', 'fugitive', 'global', 'SF6', 22800.0, 'kgCO2e/kg', 'AR4', 'DEFRA 2024', '2024');

-- Scope 3 — Supply chain (EEIO / spend-based, GHG Protocol 2023, USD-based)
-- These are approximate spend-based emission factors for initial Scope 3 estimation
INSERT OR IGNORE INTO emission_factors
  (name, category, subcategory, region, ghg_type, factor_value, factor_unit, ar_version, source, valid_from) VALUES
  ('Purchased goods — food products (spend-based)', 'scope3_cat1', 'purchased_goods', 'US', 'CO2', 0.450, 'kgCO2e/USD', 'AR5', 'USEEIO v2.1', '2023'),
  ('Purchased goods — electronic components (spend-based)', 'scope3_cat1', 'purchased_goods', 'US', 'CO2', 0.180, 'kgCO2e/USD', 'AR5', 'USEEIO v2.1', '2023'),
  ('Purchased goods — chemicals (spend-based)', 'scope3_cat1', 'purchased_goods', 'US', 'CO2', 0.390, 'kgCO2e/USD', 'AR5', 'USEEIO v2.1', '2023'),
  ('Capital goods — machinery (spend-based)', 'scope3_cat2', 'capital_goods', 'US', 'CO2', 0.200, 'kgCO2e/USD', 'AR5', 'USEEIO v2.1', '2023'),
  ('Road freight — HGV (per tonne-km)', 'scope3_cat4', 'upstream_transport', 'GB', 'CO2', 0.10230, 'kgCO2e/tkm', 'AR5', 'DEFRA 2024', '2024'),
  ('Sea freight — container ship (per tonne-km)', 'scope3_cat4', 'upstream_transport', 'global', 'CO2', 0.01630, 'kgCO2e/tkm', 'AR5', 'DEFRA 2024', '2024'),
  ('Air freight (per tonne-km)', 'scope3_cat4', 'upstream_transport', 'global', 'CO2', 0.60270, 'kgCO2e/tkm', 'AR5', 'DEFRA 2024', '2024');
