<script lang="ts">
  import { onMount } from 'svelte'
  import { activePeriod, activeOrg } from '$lib/stores/app'
  import {
    generateGri305Report, exportSourcesCsv, calculatePeriod,
    listIntensityResults, listReductions, listOdsEmissions, listAirEmissions,
  } from '$lib/tauri'
  import type { PeriodInventory, IntensityResult, Reduction, OdsEntry, AirEntry } from '$lib/tauri'

  let report = $state<Record<string, unknown> | null>(null)
  let inventory = $state<PeriodInventory | null>(null)
  let intensityResults = $state<IntensityResult[]>([])
  let reductions = $state<Reduction[]>([])
  let odsEntries = $state<OdsEntry[]>([])
  let airEntries = $state<AirEntry[]>([])
  let loading = $state(false)
  let error = $state('')
  let activeTab = $state<'305' | 'inventory'>('305')

  onMount(async () => {
    const period = $activePeriod
    if (!period) return
    loading = true
    try {
      ;[report, inventory, intensityResults, reductions, odsEntries, airEntries] = await Promise.all([
        generateGri305Report(period.id),
        calculatePeriod(period.id),
        listIntensityResults(period.id),
        listReductions(period.id),
        listOdsEmissions(period.id),
        listAirEmissions(period.id),
      ])
    } catch (e) { error = String(e) }
    finally { loading = false }
  })

  async function exportCsv() {
    const period = $activePeriod
    if (!period) return
    // Prompt user for save path via Tauri dialog would go here
    // For now export to app data dir
    try {
      await exportSourcesCsv(period.id, `c12-scope-data-${period.year}.csv`)
    } catch (e) { error = String(e) }
  }

  function fmt(n: number | null | undefined) {
    return n != null ? n.toFixed(3) : '—'
  }

  function fmtPct(n: number | null | undefined) {
    return n != null ? `${n.toFixed(1)}%` : '—'
  }

  // Extract typed fields from the raw report JSON
  function field(key: string): unknown {
    return report?.[key]
  }
</script>

<div class="p-8">
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h1 class="text-xl font-bold text-gray-100">Reports</h1>
      {#if $activePeriod}
        <p class="text-xs text-gray-500">
          {$activeOrg?.name} · {$activePeriod.year} · {$activePeriod.gwp_ar_version} GWP
        </p>
      {/if}
    </div>
    <button onclick={exportCsv}
      class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600 hover:text-gray-200">
      Export CSV
    </button>
  </div>

  <!-- Tabs -->
  <div class="mb-6 flex gap-1 rounded-xl border border-gray-800 bg-gray-900 p-1">
    <button onclick={() => activeTab = '305'}
      class="flex-1 rounded-lg py-2 text-sm font-medium transition-colors
        {activeTab === '305' ? 'bg-green-600 text-white' : 'text-gray-500 hover:text-gray-300'}">
      GRI 305 Disclosures
    </button>
    <button onclick={() => activeTab = 'inventory'}
      class="flex-1 rounded-lg py-2 text-sm font-medium transition-colors
        {activeTab === 'inventory' ? 'bg-green-600 text-white' : 'text-gray-500 hover:text-gray-300'}">
      Inventory Summary
    </button>
  </div>

  {#if loading}
    <p class="text-sm text-gray-500">Generating report…</p>
  {:else if error}
    <div class="rounded-xl border border-red-800 bg-red-950/20 p-4 text-sm text-red-400">{error}</div>
  {:else if !inventory}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
      <p class="text-sm text-gray-500">No data yet. Add emission sources to generate reports.</p>
    </div>
  {:else if activeTab === '305'}
    <!-- GRI 305 Report -->
    <div class="space-y-4">
      <!-- 305-1: Scope 1 -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-1</span>
          <span class="disclosure-title">Direct (Scope 1) GHG Emissions</span>
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div>
            <p class="label">Gross Scope 1 (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope1.gross_tco2e)}</p>
          </div>
          <div>
            <p class="label">Biogenic CO₂ (tCO₂, separate)</p>
            <p class="metric">{fmt(inventory.scope1.biogenic_co2_tco2e)}</p>
          </div>
          <div>
            <p class="label">Combined uncertainty</p>
            <p class="metric">±{fmtPct(inventory.scope1.combined_uncertainty_pct)}</p>
          </div>
        </div>
        {#if Object.keys(inventory.scope1.by_gas).length > 0}
          <div class="mt-3">
            <p class="label mb-2">By GHG type</p>
            <div class="flex flex-wrap gap-2">
              {#each Object.entries(inventory.scope1.by_gas) as [gas, val]}
                <span class="rounded-full border border-gray-700 bg-gray-800 px-3 py-1 text-xs text-gray-300">
                  {gas}: {fmt(val as number)} tCO₂e
                </span>
              {/each}
            </div>
          </div>
        {/if}
        <p class="mt-3 text-xs text-gray-600">GWP: {$activePeriod?.gwp_ar_version ?? '—'} · Boundary: {$activeOrg?.boundary_method ?? '—'}</p>
      </div>

      <!-- 305-2: Scope 2 -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-2</span>
          <span class="disclosure-title">Energy Indirect (Scope 2) GHG Emissions</span>
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div>
            <p class="label">Location-based (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope2.location_based_tco2e)}</p>
          </div>
          <div>
            <p class="label">Market-based (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope2.market_based_tco2e)}</p>
          </div>
          <div>
            <p class="label">Contractual coverage</p>
            <p class="metric">{fmtPct(inventory.scope2.contractual_coverage_pct)}</p>
          </div>
        </div>
        {#if inventory.scope2.location_based_tco2e === 0 || inventory.scope2.market_based_tco2e === 0}
          <p class="mt-3 text-xs text-yellow-500">
            Warning: GRI 305-2 requires BOTH location-based and market-based figures. At least one is missing.
          </p>
        {/if}
      </div>

      <!-- 305-3: Scope 3 -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-3</span>
          <span class="disclosure-title">Other Indirect (Scope 3) GHG Emissions</span>
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div>
            <p class="label">Gross Scope 3 (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope3.gross_tco2e)}</p>
          </div>
          <div>
            <p class="label">Upstream (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope3.upstream_tco2e)}</p>
          </div>
          <div>
            <p class="label">Downstream (tCO₂e)</p>
            <p class="metric">{fmt(inventory.scope3.downstream_tco2e)}</p>
          </div>
        </div>
        <div class="mt-3 space-y-1">
          {#each inventory.scope3.categories.filter(c => c.total_tco2e > 0 || c.is_excluded) as cat}
            <div class="flex items-center justify-between rounded-lg border border-gray-800 px-3 py-1.5">
              <div class="flex items-center gap-2">
                <span class="w-5 text-right text-xs text-gray-600">{cat.category}</span>
                <span class="text-xs text-gray-300">{cat.category_name}</span>
                {#if cat.is_excluded}
                  <span class="rounded-full border border-gray-700 px-2 py-0.5 text-[10px] text-gray-600">excluded</span>
                {/if}
              </div>
              <span class="text-xs font-semibold text-gray-300">{fmt(cat.total_tco2e)} tCO₂e</span>
            </div>
          {/each}
        </div>
        {#if inventory.scope3.excluded_categories.length > 0}
          <p class="mt-3 text-xs text-gray-500">
            Excluded: categories {inventory.scope3.excluded_categories.join(', ')} (reasons documented)
          </p>
        {/if}
      </div>

      <!-- 305-4: Intensity -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-4</span>
          <span class="disclosure-title">GHG Emissions Intensity</span>
        </div>
        {#if intensityResults.length > 0}
          <div class="space-y-3">
            {#each intensityResults as r}
              <div class="rounded-lg border border-gray-800 px-4 py-3">
                <div class="flex items-center justify-between">
                  <p class="text-sm font-semibold text-gray-200">{r.metric_name}</p>
                  <p class="text-lg font-bold text-gray-100">{r.intensity_ratio.toFixed(4)} <span class="text-xs font-normal text-gray-500">tCO₂e / {r.metric_unit}</span></p>
                </div>
                <p class="mt-1 text-xs text-gray-500">
                  {r.total_emissions_tco2e.toFixed(2)} tCO₂e ÷ {r.metric_value} {r.metric_unit} ·
                  Scopes: {[r.includes_scope1 && '1', r.includes_scope2 && '2', r.includes_scope3 && '3'].filter(Boolean).join('+')}
                </p>
                {#if r.scope3_intensity_ratio != null}
                  <p class="text-xs text-gray-600">Scope 3 only: {r.scope3_intensity_ratio.toFixed(4)} tCO₂e / {r.metric_unit}</p>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-xs text-gray-500">No intensity metrics defined. Add one in Settings → Supplemental.</p>
          <p class="mt-1 text-xs text-gray-600">Total Scope 1+2: {fmt(inventory.scope1_scope2_tco2e)} tCO₂e</p>
        {/if}
      </div>

      <!-- 305-5: Reductions -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-5</span>
          <span class="disclosure-title">Reduction of GHG Emissions</span>
        </div>
        {#if reductions.length > 0}
          <div class="space-y-2">
            {#each reductions as r}
              <div class="rounded-lg border border-gray-800 px-4 py-3">
                <div class="flex items-center justify-between">
                  <p class="text-sm text-gray-300">{r.methodology}</p>
                  <p class="text-sm font-semibold text-green-400">{r.reduction_tco2e.toFixed(2)} tCO₂e ({r.reduction_pct.toFixed(1)}%)</p>
                </div>
                <p class="mt-1 text-xs text-gray-600">Baseline {r.baseline_year}: {r.baseline_tco2e.toFixed(2)} tCO₂e → {r.current_tco2e.toFixed(2)} tCO₂e this period</p>
              </div>
            {/each}
          </div>
          <p class="mt-3 text-xs text-gray-600">Reductions from outsourcing and production cuts excluded (GRI 305-5 requirement).</p>
        {:else}
          <p class="text-xs text-gray-500">No reductions recorded. Add them in Settings → Supplemental.</p>
        {/if}
      </div>

      <!-- 305-6: ODS -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-6</span>
          <span class="disclosure-title">Emissions of Ozone-Depleting Substances</span>
        </div>
        {#if odsEntries.length > 0}
          <div class="overflow-hidden rounded-lg border border-gray-800">
            <table class="w-full text-xs">
              <thead class="border-b border-gray-800 bg-gray-800/40">
                <tr class="text-left text-gray-500">
                  <th class="px-3 py-2">Substance</th>
                  <th class="px-3 py-2">Production (t)</th>
                  <th class="px-3 py-2">Imports (t)</th>
                  <th class="px-3 py-2">Exports (t)</th>
                  <th class="px-3 py-2">CFC-11 eq (t)</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each odsEntries as e}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2 font-medium">{e.substance}</td>
                    <td class="px-3 py-2">{e.production_metric_tons}</td>
                    <td class="px-3 py-2">{e.imports_metric_tons}</td>
                    <td class="px-3 py-2">{e.exports_metric_tons}</td>
                    <td class="px-3 py-2 font-semibold">{e.cfc11_equivalent}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No ODS entries. Add them in Settings → Supplemental.</p>
        {/if}
      </div>

      <!-- 305-7: Air emissions -->
      <div class="disclosure-card">
        <div class="disclosure-header">
          <span class="disclosure-code">GRI 305-7</span>
          <span class="disclosure-title">Nitrogen Oxides, Sulfur Oxides and Other Air Emissions</span>
        </div>
        {#if airEntries.length > 0}
          <div class="overflow-hidden rounded-lg border border-gray-800">
            <table class="w-full text-xs">
              <thead class="border-b border-gray-800 bg-gray-800/40">
                <tr class="text-left text-gray-500">
                  <th class="px-3 py-2">Type</th>
                  <th class="px-3 py-2">Substance</th>
                  <th class="px-3 py-2">Value (metric t)</th>
                  <th class="px-3 py-2">Method</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each airEntries as e}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2 font-medium">{e.emission_type}</td>
                    <td class="px-3 py-2 text-gray-500">{e.substance ?? '—'}</td>
                    <td class="px-3 py-2 font-semibold">{e.value_metric_tons}</td>
                    <td class="px-3 py-2 text-gray-500">{e.measurement_method.replace('_', ' ')}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No air emissions recorded. Add them in Settings → Supplemental.</p>
        {/if}
      </div>
    </div>

  {:else}
    <!-- Inventory Summary -->
    <div class="space-y-4">
      <div class="rounded-xl border border-green-900/50 bg-green-950/20 p-5">
        <p class="text-xs font-semibold uppercase tracking-wider text-green-600">Total GHG Inventory</p>
        <p class="mt-1 text-3xl font-black text-green-400">
          {fmt(inventory.total_tco2e)} <span class="text-base font-normal text-green-700">tCO₂e</span>
        </p>
        <p class="mt-1 text-xs text-gray-500">Scope 1+2+3 · {$activePeriod?.gwp_ar_version} GWP · {$activePeriod?.year}</p>
      </div>

      <div class="grid gap-4 sm:grid-cols-3">
        <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
          <p class="label">Scope 1</p>
          <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope1.gross_tco2e)}</p>
          <p class="text-xs text-gray-500">tCO₂e · {inventory.scope1.sources.length} sources</p>
        </div>
        <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
          <p class="label">Scope 2 (location)</p>
          <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope2.location_based_tco2e)}</p>
          <p class="text-xs text-gray-500">tCO₂e · market: {fmt(inventory.scope2.market_based_tco2e)}</p>
        </div>
        <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
          <p class="label">Scope 3</p>
          <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope3.gross_tco2e)}</p>
          <p class="text-xs text-gray-500">tCO₂e · upstream + downstream</p>
        </div>
      </div>

      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <h2 class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">ISO 14064-1 Inventory Elements</h2>
        <div class="space-y-2">
          {#each [
            { label: 'Organizational boundary', value: $activeOrg?.boundary_method?.replace(/_/g, ' ') ?? '—' },
            { label: 'Reporting period', value: `${$activePeriod?.start_date} to ${$activePeriod?.end_date}` },
            { label: 'GWP version (IPCC)', value: $activePeriod?.gwp_ar_version ?? '—' },
            { label: 'Base year', value: $activeOrg?.base_year?.toString() ?? '—' },
            { label: 'Reporting status', value: $activePeriod?.status ?? '—' },
          ] as item}
            <div class="flex items-center justify-between border-b border-gray-800 pb-2 last:border-0">
              <span class="text-xs text-gray-500">{item.label}</span>
              <span class="text-xs text-gray-300">{item.value}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .label { @apply mb-0.5 block text-xs font-medium text-gray-500; }
  .metric { @apply text-xl font-bold text-gray-100; }

  .disclosure-card {
    @apply rounded-xl border border-gray-800 bg-gray-900 p-5;
  }

  .disclosure-header {
    @apply mb-4 flex items-center gap-3;
  }

  .disclosure-code {
    @apply rounded border border-green-800/60 bg-green-950/30 px-2 py-0.5 font-mono text-xs text-green-400;
  }

  .disclosure-title {
    @apply text-sm font-semibold text-gray-200;
  }
</style>
