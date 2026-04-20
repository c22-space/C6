<script lang="ts">
  import { onMount } from 'svelte'
  import { activeOrg, activePeriod } from '$lib/stores/app'
  import {
    listOrgs, updateOrg, listPeriods, createPeriod, listEntities, createEntity, getAuditLog,
    listIntensityResults, saveIntensityMetric, deleteIntensityResult,
    listReductions, createReduction, deleteReduction,
    listOdsEmissions, createOdsEmission, deleteOdsEmission,
    listAirEmissions, createAirEmission, deleteAirEmission,
  } from '$lib/tauri'
  import type { Entity, ReportingPeriod, IntensityResult, Reduction, OdsEntry, AirEntry } from '$lib/tauri'

  let periods = $state<ReportingPeriod[]>([])
  let entities = $state<Entity[]>([])
  let auditLog = $state<Record<string, unknown>[]>([])
  let activeTab = $state<'org' | 'periods' | 'entities' | 'supplemental' | 'audit' | 'enterprise'>('org')

  // Supplemental data
  let intensityResults = $state<IntensityResult[]>([])
  let reductions = $state<Reduction[]>([])
  let odsEntries = $state<OdsEntry[]>([])
  let airEntries = $state<AirEntry[]>([])

  // Intensity form
  let intForm = $state({ metric_name: '', metric_value: 0, metric_unit: '', includes_scope1: true, includes_scope2: true, includes_scope3: false })
  let showIntForm = $state(false)

  // Reduction form
  let redForm = $state({ baseline_year: new Date().getFullYear() - 1, baseline_tco2e: 0, current_tco2e: 0, methodology: '' })
  let showRedForm = $state(false)

  // ODS form
  let odsForm = $state({ substance: '', production_metric_tons: 0, imports_metric_tons: 0, exports_metric_tons: 0, cfc11_equivalent: 0 })
  let showOdsForm = $state(false)

  // Air form
  let airForm = $state({ emission_type: 'NOx', substance: '', value_metric_tons: 0, measurement_method: 'estimation' })
  let showAirForm = $state(false)
  let error = $state('')
  let success = $state('')

  // Org edit form
  let orgForm = $state({
    name: '',
    boundary_method: 'operational_control',
    base_year: new Date().getFullYear() - 1,
  })

  // New period form
  let newPeriod = $state({
    year: new Date().getFullYear(),
    gwp_ar_version: 'AR6',
  })
  let showPeriodForm = $state(false)

  // New entity form
  let newEntity = $state({
    name: '',
    type: 'subsidiary' as const,
    ownership_pct: 100,
    country_code: '',
  })
  let showEntityForm = $state(false)

  const boundaryMethods = [
    { value: 'operational_control', label: 'Operational Control' },
    { value: 'financial_control',   label: 'Financial Control' },
    { value: 'equity_share',        label: 'Equity Share' },
  ]

  const entityTypes = ['parent', 'subsidiary', 'facility', 'jv', 'branch']

  onMount(async () => {
    const org = $activeOrg
    if (!org) return
    orgForm.name = org.name
    orgForm.boundary_method = org.boundary_method
    orgForm.base_year = org.base_year ?? new Date().getFullYear() - 1
    periods = await listPeriods(org.id)
    entities = await listEntities(org.id)
  })

  async function loadSupplemental() {
    const period = $activePeriod
    if (!period) return
    ;[intensityResults, reductions, odsEntries, airEntries] = await Promise.all([
      listIntensityResults(period.id),
      listReductions(period.id),
      listOdsEmissions(period.id),
      listAirEmissions(period.id),
    ])
  }

  async function saveIntensity() {
    const period = $activePeriod
    if (!period || !intForm.metric_name.trim() || intForm.metric_value <= 0) return
    error = ''
    try {
      await saveIntensityMetric({ period_id: period.id, ...intForm, metric_name: intForm.metric_name.trim() })
      intensityResults = await listIntensityResults(period.id)
      showIntForm = false
      intForm = { metric_name: '', metric_value: 0, metric_unit: '', includes_scope1: true, includes_scope2: true, includes_scope3: false }
    } catch (e) { error = String(e) }
  }

  async function removeIntensity(metric_name: string) {
    const period = $activePeriod
    if (!period) return
    await deleteIntensityResult(period.id, metric_name)
    intensityResults = await listIntensityResults(period.id)
  }

  async function saveReduction() {
    const period = $activePeriod
    if (!period || !redForm.methodology.trim()) return
    error = ''
    try {
      await createReduction({ period_id: period.id, ...redForm, methodology: redForm.methodology.trim() })
      reductions = await listReductions(period.id)
      showRedForm = false
      redForm = { baseline_year: new Date().getFullYear() - 1, baseline_tco2e: 0, current_tco2e: 0, methodology: '' }
    } catch (e) { error = String(e) }
  }

  async function removeReduction(id: number) {
    await deleteReduction(id)
    const period = $activePeriod
    if (period) reductions = await listReductions(period.id)
  }

  async function saveOds() {
    const period = $activePeriod
    if (!period || !odsForm.substance.trim()) return
    error = ''
    try {
      await createOdsEmission({ period_id: period.id, ...odsForm, substance: odsForm.substance.trim() })
      odsEntries = await listOdsEmissions(period.id)
      showOdsForm = false
      odsForm = { substance: '', production_metric_tons: 0, imports_metric_tons: 0, exports_metric_tons: 0, cfc11_equivalent: 0 }
    } catch (e) { error = String(e) }
  }

  async function removeOds(id: number) {
    await deleteOdsEmission(id)
    const period = $activePeriod
    if (period) odsEntries = await listOdsEmissions(period.id)
  }

  async function saveAir() {
    const period = $activePeriod
    if (!period || airForm.value_metric_tons <= 0) return
    error = ''
    try {
      await createAirEmission({ period_id: period.id, ...airForm, substance: airForm.substance || null })
      airEntries = await listAirEmissions(period.id)
      showAirForm = false
      airForm = { emission_type: 'NOx', substance: '', value_metric_tons: 0, measurement_method: 'estimation' }
    } catch (e) { error = String(e) }
  }

  async function removeAir(id: number) {
    await deleteAirEmission(id)
    const period = $activePeriod
    if (period) airEntries = await listAirEmissions(period.id)
  }

  async function saveOrg() {
    const org = $activeOrg
    if (!org) return
    error = ''; success = ''
    try {
      await updateOrg({
        id: org.id,
        name: orgForm.name,
        boundary_method: orgForm.boundary_method,
        base_year: orgForm.base_year,
      })
      activeOrg.update(o => o ? { ...o, name: orgForm.name, boundary_method: orgForm.boundary_method as any, base_year: orgForm.base_year } : o)
      success = 'Organisation updated.'
    } catch (e) { error = String(e) }
  }

  async function addPeriod() {
    const org = $activeOrg
    if (!org) return
    error = ''; success = ''
    try {
      const period = await createPeriod({
        org_id: org.id,
        year: newPeriod.year,
        start_date: `${newPeriod.year}-01-01`,
        end_date: `${newPeriod.year}-12-31`,
        gwp_ar_version: newPeriod.gwp_ar_version,
      })
      periods = [...periods, period]
      showPeriodForm = false
      success = `Period ${newPeriod.year} created.`
    } catch (e) { error = String(e) }
  }

  async function addEntity() {
    const org = $activeOrg
    if (!org) return
    error = ''; success = ''
    try {
      const entity = await createEntity({
        org_id: org.id,
        name: newEntity.name,
        type: newEntity.type,
        ownership_pct: newEntity.ownership_pct,
        is_financially_controlled: true,
        is_operationally_controlled: true,
        country_code: newEntity.country_code || null,
      })
      entities = [...entities, entity]
      showEntityForm = false
      newEntity = { name: '', type: 'subsidiary', ownership_pct: 100, country_code: '' }
      success = 'Entity added.'
    } catch (e) { error = String(e) }
  }

  async function loadAuditLog() {
    const org = $activeOrg
    if (!org) return
    try {
      // Show recent audit entries for the organization
      auditLog = await getAuditLog('organizations', org.id)
    } catch (e) { error = String(e) }
  }

  function setActivePeriod(p: ReportingPeriod) {
    activePeriod.set(p)
    success = `Active period set to ${p.year}.`
  }

  function fmt(ts: number) {
    return new Date(ts * 1000).toLocaleString()
  }
</script>

<div class="p-8">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-100">Settings</h1>
    <p class="text-xs text-gray-500">Organisation configuration, periods, entities, and audit trail</p>
  </div>

  <!-- Tabs -->
  <div class="mb-6 flex flex-wrap gap-1 rounded-xl border border-gray-800 bg-gray-900 p-1">
    {#each [
      { key: 'org',          label: 'Organisation' },
      { key: 'periods',      label: 'Periods' },
      { key: 'entities',     label: 'Entities' },
      { key: 'supplemental', label: 'Supplemental' },
      { key: 'audit',        label: 'Audit Trail' },
      { key: 'enterprise',   label: 'Enterprise' },
    ] as tab}
      <button onclick={() => { activeTab = tab.key as any; if (tab.key === 'supplemental') loadSupplemental() }}
        class="rounded-lg px-4 py-2 text-sm font-medium transition-colors
          {activeTab === tab.key ? 'bg-green-600 text-white' : 'text-gray-500 hover:text-gray-300'}">
        {tab.label}
      </button>
    {/each}
  </div>

  {#if error}<div class="mb-4 rounded-lg border border-red-800 bg-red-950/20 p-3 text-xs text-red-400">{error}</div>{/if}
  {#if success}<div class="mb-4 rounded-lg border border-green-800 bg-green-950/20 p-3 text-xs text-green-400">{success}</div>{/if}

  <!-- Organisation tab -->
  {#if activeTab === 'org'}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
      <h2 class="mb-4 text-sm font-semibold text-gray-200">Organisation Details</h2>
      <div class="space-y-4">
        <div>
          <label class="label">Organisation name</label>
          <input bind:value={orgForm.name} type="text" class="input" />
        </div>
        <div>
          <label class="label">Organisational boundary method <span class="text-gray-600">(ISO 14064-1 §5.2)</span></label>
          <div class="space-y-2">
            {#each boundaryMethods as m}
              <label class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors
                {orgForm.boundary_method === m.value ? 'border-green-700 bg-green-950/20' : 'border-gray-800 hover:border-gray-700'}">
                <input type="radio" bind:group={orgForm.boundary_method} value={m.value} />
                <span class="text-sm text-gray-200">{m.label}</span>
              </label>
            {/each}
          </div>
        </div>
        <div>
          <label class="label">Base year</label>
          <input bind:value={orgForm.base_year} type="number" min="2000" max="2030" class="input w-32" />
        </div>
      </div>
      <button onclick={saveOrg} class="mt-5 rounded-lg bg-green-600 px-5 py-2 text-sm font-semibold text-white hover:bg-green-700">
        Save changes
      </button>
    </div>

  <!-- Periods tab -->
  {:else if activeTab === 'periods'}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-200">Reporting Periods</h2>
        <button onclick={() => showPeriodForm = !showPeriodForm}
          class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
          + New period
        </button>
      </div>

      {#if showPeriodForm}
        <div class="mb-5 rounded-lg border border-gray-700 p-4">
          <div class="grid gap-3 sm:grid-cols-2">
            <div>
              <label class="label">Year</label>
              <input bind:value={newPeriod.year} type="number" min="2000" max="2030" class="input" />
            </div>
            <div>
              <label class="label">GWP version</label>
              <select bind:value={newPeriod.gwp_ar_version} class="input">
                <option value="AR6">AR6 (2021) — Recommended</option>
                <option value="AR5">AR5 (2013)</option>
                <option value="AR4">AR4 (2007)</option>
              </select>
            </div>
          </div>
          <div class="mt-3 flex gap-2">
            <button onclick={addPeriod} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Create</button>
            <button onclick={() => showPeriodForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
          </div>
        </div>
      {/if}

      <div class="space-y-2">
        {#each periods as p}
          <div class="flex items-center justify-between rounded-lg border border-gray-800 px-4 py-3
            {$activePeriod?.id === p.id ? 'border-green-800/50 bg-green-950/10' : ''}">
            <div>
              <p class="text-sm font-medium text-gray-200">{p.year} · {p.gwp_ar_version}</p>
              <p class="text-xs text-gray-500">{p.start_date} → {p.end_date} · {p.status}</p>
            </div>
            {#if $activePeriod?.id === p.id}
              <span class="text-xs text-green-500">Active</span>
            {:else}
              <button onclick={() => setActivePeriod(p)}
                class="rounded-lg border border-gray-700 px-3 py-1 text-xs text-gray-400 hover:border-gray-600">
                Set active
              </button>
            {/if}
          </div>
        {:else}
          <p class="text-sm text-gray-500">No periods yet.</p>
        {/each}
      </div>
    </div>

  <!-- Entities tab -->
  {:else if activeTab === 'entities'}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-200">Legal Entities / Facilities</h2>
        <button onclick={() => showEntityForm = !showEntityForm}
          class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
          + Add entity
        </button>
      </div>

      {#if showEntityForm}
        <div class="mb-5 rounded-lg border border-gray-700 p-4">
          <div class="grid gap-3 sm:grid-cols-2">
            <div>
              <label class="label">Name</label>
              <input bind:value={newEntity.name} type="text" placeholder="Acme Corp UK Ltd" class="input" />
            </div>
            <div>
              <label class="label">Type</label>
              <select bind:value={newEntity.type} class="input">
                {#each entityTypes as t}<option>{t}</option>{/each}
              </select>
            </div>
            <div>
              <label class="label">Ownership % (for equity share)</label>
              <input bind:value={newEntity.ownership_pct} type="number" min="0" max="100" step="0.1" class="input" />
            </div>
            <div>
              <label class="label">Country code</label>
              <input bind:value={newEntity.country_code} type="text" placeholder="GB, US, AU…" maxlength="2" class="input" />
            </div>
          </div>
          <div class="mt-3 flex gap-2">
            <button onclick={addEntity} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Add</button>
            <button onclick={() => showEntityForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
          </div>
        </div>
      {/if}

      <div class="space-y-2">
        {#each entities as e}
          <div class="flex items-center justify-between rounded-lg border border-gray-800 px-4 py-3">
            <div>
              <p class="text-sm font-medium text-gray-200">{e.name}</p>
              <p class="text-xs text-gray-500">
                {e.type}
                {e.ownership_pct != null ? ` · ${e.ownership_pct}% owned` : ''}
                {e.country_code ? ` · ${e.country_code}` : ''}
              </p>
            </div>
            <span class="rounded-full border border-gray-700 px-2 py-0.5 text-[10px] text-gray-500">{e.type}</span>
          </div>
        {:else}
          <p class="text-sm text-gray-500">No entities yet.</p>
        {/each}
      </div>
    </div>

  <!-- Supplemental Emissions tab (GRI 305-4 through 305-7) -->
  {:else if activeTab === 'supplemental'}
    <div class="space-y-6">

      <!-- 305-4: Intensity Ratios -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h2 class="text-sm font-semibold text-gray-200">Emissions Intensity (GRI 305-4)</h2>
            <p class="text-xs text-gray-500">tCO₂e per unit of an activity metric (e.g. revenue, employees, m² floor area)</p>
          </div>
          <button onclick={() => showIntForm = !showIntForm}
            class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
            + Add metric
          </button>
        </div>

        {#if showIntForm}
          <div class="mb-5 rounded-lg border border-gray-700 p-4">
            <div class="grid gap-3 sm:grid-cols-2">
              <div class="sm:col-span-2">
                <label class="label">Metric name (e.g. Revenue USD, Units Produced, Employees)</label>
                <input bind:value={intForm.metric_name} type="text" placeholder="Revenue (USD M)" class="input" />
              </div>
              <div>
                <label class="label">Metric value</label>
                <input bind:value={intForm.metric_value} type="number" step="0.01" min="0" class="input" />
              </div>
              <div>
                <label class="label">Unit</label>
                <input bind:value={intForm.metric_unit} type="text" placeholder="USD M, units, FTE…" class="input" />
              </div>
            </div>
            <div class="mt-3 flex flex-wrap gap-4">
              <label class="flex items-center gap-2 text-xs text-gray-400">
                <input type="checkbox" bind:checked={intForm.includes_scope1} /> Include Scope 1
              </label>
              <label class="flex items-center gap-2 text-xs text-gray-400">
                <input type="checkbox" bind:checked={intForm.includes_scope2} /> Include Scope 2
              </label>
              <label class="flex items-center gap-2 text-xs text-gray-400">
                <input type="checkbox" bind:checked={intForm.includes_scope3} /> Include Scope 3
              </label>
            </div>
            <div class="mt-3 flex gap-2">
              <button onclick={saveIntensity} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
              <button onclick={() => showIntForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
            </div>
          </div>
        {/if}

        {#if intensityResults.length > 0}
          <div class="overflow-hidden rounded-lg border border-gray-800">
            <table class="w-full text-xs">
              <thead class="border-b border-gray-800 bg-gray-800/40">
                <tr class="text-left text-gray-500">
                  <th class="px-3 py-2">Metric</th>
                  <th class="px-3 py-2">Value</th>
                  <th class="px-3 py-2">Ratio (tCO₂e/unit)</th>
                  <th class="px-3 py-2">Scopes</th>
                  <th class="px-3 py-2"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each intensityResults as r}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2">{r.metric_name}</td>
                    <td class="px-3 py-2">{r.metric_value} {r.metric_unit}</td>
                    <td class="px-3 py-2 font-semibold">{r.intensity_ratio.toFixed(4)}</td>
                    <td class="px-3 py-2 text-gray-500">
                      {[r.includes_scope1 && 'S1', r.includes_scope2 && 'S2', r.includes_scope3 && 'S3'].filter(Boolean).join('+')}
                    </td>
                    <td class="px-3 py-2">
                      <button onclick={() => removeIntensity(r.metric_name)} class="text-red-500 hover:text-red-400">✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No intensity metrics defined yet.</p>
        {/if}
      </div>

      <!-- 305-5: Reductions -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h2 class="text-sm font-semibold text-gray-200">Emissions Reductions (GRI 305-5)</h2>
            <p class="text-xs text-gray-500">Reductions from specific initiatives vs baseline. Excludes outsourcing and production cuts.</p>
          </div>
          <button onclick={() => showRedForm = !showRedForm}
            class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
            + Add
          </button>
        </div>

        {#if showRedForm}
          <div class="mb-5 rounded-lg border border-gray-700 p-4">
            <div class="grid gap-3 sm:grid-cols-2">
              <div>
                <label class="label">Baseline year</label>
                <input bind:value={redForm.baseline_year} type="number" min="2000" max="2030" class="input" />
              </div>
              <div>
                <label class="label">Baseline emissions (tCO₂e)</label>
                <input bind:value={redForm.baseline_tco2e} type="number" step="0.01" min="0" class="input" />
              </div>
              <div>
                <label class="label">Current period emissions (tCO₂e)</label>
                <input bind:value={redForm.current_tco2e} type="number" step="0.01" min="0" class="input" />
              </div>
              <div>
                <label class="label">Methodology</label>
                <input bind:value={redForm.methodology} type="text" placeholder="e.g. Building electrification, LED retrofit" class="input" />
              </div>
            </div>
            <div class="mt-3 flex gap-2">
              <button onclick={saveReduction} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
              <button onclick={() => showRedForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
            </div>
          </div>
        {/if}

        {#if reductions.length > 0}
          <div class="overflow-hidden rounded-lg border border-gray-800">
            <table class="w-full text-xs">
              <thead class="border-b border-gray-800 bg-gray-800/40">
                <tr class="text-left text-gray-500">
                  <th class="px-3 py-2">Baseline year</th>
                  <th class="px-3 py-2">Baseline (tCO₂e)</th>
                  <th class="px-3 py-2">Current (tCO₂e)</th>
                  <th class="px-3 py-2">Reduction</th>
                  <th class="px-3 py-2">Methodology</th>
                  <th class="px-3 py-2"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each reductions as r}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2">{r.baseline_year}</td>
                    <td class="px-3 py-2">{r.baseline_tco2e.toFixed(2)}</td>
                    <td class="px-3 py-2">{r.current_tco2e.toFixed(2)}</td>
                    <td class="px-3 py-2 font-semibold text-green-400">{r.reduction_tco2e.toFixed(2)} tCO₂e ({r.reduction_pct.toFixed(1)}%)</td>
                    <td class="px-3 py-2 text-gray-500">{r.methodology}</td>
                    <td class="px-3 py-2">
                      <button onclick={() => removeReduction(r.id)} class="text-red-500 hover:text-red-400">✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No reductions recorded yet.</p>
        {/if}
      </div>

      <!-- 305-6: ODS Emissions -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h2 class="text-sm font-semibold text-gray-200">Ozone-Depleting Substances (GRI 305-6)</h2>
            <p class="text-xs text-gray-500">Production, imports, and exports of ODS in metric tonnes and CFC-11 equivalent.</p>
          </div>
          <button onclick={() => showOdsForm = !showOdsForm}
            class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
            + Add
          </button>
        </div>

        {#if showOdsForm}
          <div class="mb-5 rounded-lg border border-gray-700 p-4">
            <div class="grid gap-3 sm:grid-cols-2">
              <div class="sm:col-span-2">
                <label class="label">Substance (e.g. R-22, R-410A, Halon-1301)</label>
                <input bind:value={odsForm.substance} type="text" class="input" />
              </div>
              <div>
                <label class="label">Production (metric tonnes)</label>
                <input bind:value={odsForm.production_metric_tons} type="number" step="0.001" min="0" class="input" />
              </div>
              <div>
                <label class="label">Imports (metric tonnes)</label>
                <input bind:value={odsForm.imports_metric_tons} type="number" step="0.001" min="0" class="input" />
              </div>
              <div>
                <label class="label">Exports (metric tonnes)</label>
                <input bind:value={odsForm.exports_metric_tons} type="number" step="0.001" min="0" class="input" />
              </div>
              <div>
                <label class="label">CFC-11 equivalent (metric tonnes)</label>
                <input bind:value={odsForm.cfc11_equivalent} type="number" step="0.001" min="0" class="input" />
              </div>
            </div>
            <div class="mt-3 flex gap-2">
              <button onclick={saveOds} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
              <button onclick={() => showOdsForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
            </div>
          </div>
        {/if}

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
                  <th class="px-3 py-2"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each odsEntries as e}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2 font-medium">{e.substance}</td>
                    <td class="px-3 py-2">{e.production_metric_tons}</td>
                    <td class="px-3 py-2">{e.imports_metric_tons}</td>
                    <td class="px-3 py-2">{e.exports_metric_tons}</td>
                    <td class="px-3 py-2">{e.cfc11_equivalent}</td>
                    <td class="px-3 py-2">
                      <button onclick={() => removeOds(e.id)} class="text-red-500 hover:text-red-400">✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No ODS entries recorded yet.</p>
        {/if}
      </div>

      <!-- 305-7: Air Emissions -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h2 class="text-sm font-semibold text-gray-200">Air Emissions (GRI 305-7)</h2>
            <p class="text-xs text-gray-500">NOx, SOx, VOC, particulate matter, and other significant air emissions in metric tonnes.</p>
          </div>
          <button onclick={() => showAirForm = !showAirForm}
            class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
            + Add
          </button>
        </div>

        {#if showAirForm}
          <div class="mb-5 rounded-lg border border-gray-700 p-4">
            <div class="grid gap-3 sm:grid-cols-2">
              <div>
                <label class="label">Emission type</label>
                <select bind:value={airForm.emission_type} class="input">
                  {#each ['NOx','SOx','VOC','PM','HAP','other'] as t}<option>{t}</option>{/each}
                </select>
              </div>
              <div>
                <label class="label">Substance (optional, e.g. SO₂, PM2.5)</label>
                <input bind:value={airForm.substance} type="text" placeholder="Optional" class="input" />
              </div>
              <div>
                <label class="label">Value (metric tonnes)</label>
                <input bind:value={airForm.value_metric_tons} type="number" step="0.001" min="0" class="input" />
              </div>
              <div>
                <label class="label">Measurement method</label>
                <select bind:value={airForm.measurement_method} class="input">
                  <option value="direct_measurement">Direct measurement</option>
                  <option value="estimation">Estimation</option>
                  <option value="balance">Mass balance</option>
                </select>
              </div>
            </div>
            <div class="mt-3 flex gap-2">
              <button onclick={saveAir} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
              <button onclick={() => showAirForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">Cancel</button>
            </div>
          </div>
        {/if}

        {#if airEntries.length > 0}
          <div class="overflow-hidden rounded-lg border border-gray-800">
            <table class="w-full text-xs">
              <thead class="border-b border-gray-800 bg-gray-800/40">
                <tr class="text-left text-gray-500">
                  <th class="px-3 py-2">Type</th>
                  <th class="px-3 py-2">Substance</th>
                  <th class="px-3 py-2">Value (t)</th>
                  <th class="px-3 py-2">Method</th>
                  <th class="px-3 py-2"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each airEntries as e}
                  <tr class="text-gray-300">
                    <td class="px-3 py-2 font-medium">{e.emission_type}</td>
                    <td class="px-3 py-2 text-gray-500">{e.substance ?? '—'}</td>
                    <td class="px-3 py-2">{e.value_metric_tons}</td>
                    <td class="px-3 py-2 text-gray-500">{e.measurement_method.replace('_', ' ')}</td>
                    <td class="px-3 py-2">
                      <button onclick={() => removeAir(e.id)} class="text-red-500 hover:text-red-400">✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-xs text-gray-500">No air emissions recorded yet.</p>
        {/if}
      </div>
    </div>

  <!-- Audit Trail tab -->
  {:else if activeTab === 'audit'}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h2 class="text-sm font-semibold text-gray-200">Immutable Audit Trail</h2>
          <p class="text-xs text-gray-500">ISO 14064-1 §5.5 · All changes are logged and cannot be deleted</p>
        </div>
        <button onclick={loadAuditLog}
          class="rounded-lg border border-gray-700 px-3 py-1.5 text-xs text-gray-400 hover:border-gray-600">
          Load log
        </button>
      </div>

      {#if auditLog.length === 0}
        <p class="text-sm text-gray-500">Click "Load log" to view the audit trail.</p>
      {:else}
        <div class="overflow-hidden rounded-lg border border-gray-800">
          <table class="w-full text-xs">
            <thead class="border-b border-gray-800 bg-gray-800/40">
              <tr class="text-left text-gray-500">
                <th class="px-3 py-2">Timestamp</th>
                <th class="px-3 py-2">Table</th>
                <th class="px-3 py-2">Action</th>
                <th class="px-3 py-2">Field</th>
                <th class="px-3 py-2">Old</th>
                <th class="px-3 py-2">New</th>
                <th class="px-3 py-2">Reason</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-800">
              {#each auditLog as entry}
                <tr class="text-gray-400 hover:bg-gray-800/20">
                  <td class="px-3 py-2 font-mono">{fmt(entry.timestamp as number)}</td>
                  <td class="px-3 py-2">{entry.table_name as string}</td>
                  <td class="px-3 py-2">
                    <span class="{entry.action === 'DELETE' ? 'text-red-400' : entry.action === 'UPDATE' ? 'text-yellow-400' : 'text-green-400'}">
                      {entry.action as string}
                    </span>
                  </td>
                  <td class="px-3 py-2">{(entry.field_name as string) ?? '—'}</td>
                  <td class="px-3 py-2 text-gray-600">{(entry.old_value as string) ?? '—'}</td>
                  <td class="px-3 py-2">{(entry.new_value as string) ?? '—'}</td>
                  <td class="px-3 py-2 text-gray-500">{(entry.reason as string) ?? '—'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

  <!-- Enterprise tab -->
  {:else if activeTab === 'enterprise'}
    <div class="space-y-4">
      <!-- Trial / connection status -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <h2 class="mb-1 text-sm font-semibold text-gray-200">Enterprise Access</h2>
        <p class="mb-5 text-xs text-gray-500">
          Cloud sync, multi-user access, SSO, and priority support.
        </p>

        <!-- Free trial CTA -->
        <div class="rounded-lg border border-green-800/50 bg-green-950/20 p-4">
          <div class="mb-3 flex items-start justify-between gap-4">
            <div>
              <p class="text-sm font-semibold text-green-400">14-day free trial — no credit card required</p>
              <p class="mt-1 text-xs text-gray-400">
                Try all Enterprise features: cloud sync, 5 team seats, and SSO.
                Your local data stays on your device throughout.
              </p>
            </div>
          </div>
          <button class="rounded-lg bg-green-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-green-700">
            Start free trial →
          </button>
          <p class="mt-2 text-[10px] text-gray-600">
            Opens browser for SSO login. After login, trial activates immediately.
            Upgrade to paid at any time — $20/seat/month.
          </p>
        </div>

        <div class="mt-4 rounded-lg border border-gray-800 p-4">
          <div class="mb-2 flex items-center justify-between">
            <p class="text-sm font-medium text-gray-300">Already have a licence?</p>
            <span class="rounded-full border border-gray-700 px-2 py-0.5 text-[10px] text-gray-500">Not connected</span>
          </div>
          <button class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600 hover:text-gray-200">
            Connect to team →
          </button>
        </div>
      </div>

      <!-- Feature list -->
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <p class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">Included in Enterprise & Trial</p>
        <div class="space-y-2">
          {#each [
            'Multi-user access — admin, editor, and viewer roles',
            'Invite team members by email, manage seats',
            'Cloud sync — real-time, across all your devices',
            'Single sign-on — Okta, Azure AD, Google Workspace',
            'Priority support with SLA',
            '14-day free trial · 5 seats · no card required',
          ] as f}
            <div class="flex items-start gap-2 text-xs">
              <span class="mt-0.5 text-green-500">✓</span>
              <span class="text-gray-400">{f}</span>
            </div>
          {/each}
        </div>
      </div>

      <p class="text-xs text-gray-600">
        Need a custom quote or on-premise deployment?
        <a href="https://c22.space/hire" target="_blank" rel="noopener"
          class="text-gray-400 underline hover:text-gray-200">Contact c22 →</a>
      </p>
    </div>
  {/if}
</div>

<style>
  .label { @apply mb-1 block text-xs font-medium text-gray-400; }
  .input { @apply w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none; }
</style>
