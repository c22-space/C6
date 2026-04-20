<script lang="ts">
  import { onMount } from 'svelte'
  import { activePeriod, activeOrg } from '$lib/stores/app'
  import { listSources, createSource, deleteSource, listEmissionFactors, listEntities } from '$lib/tauri'
  import type { EmissionSource } from '$lib/tauri'

  let locationSources = $state<EmissionSource[]>([])
  let marketSources = $state<EmissionSource[]>([])
  let entities = $state<{ id: number; name: string }[]>([])
  let showForm = $state<'location' | 'market' | null>(null)
  let error = $state('')

  // Location-based form
  let loc = $state({
    entity_id: 0,
    category_name: '',
    activity_value: 0,
    activity_unit: 'kWh',
    activity_source: 'Meter',
    emission_factor_value: 0,
    emission_factor_unit: 'kgCO2e/kWh',
    emission_factor_source: 'National grid average',
    uncertainty_pct: 5,
    notes: '',
  })

  // Market-based form
  let mkt = $state({
    entity_id: 0,
    category_name: '',
    activity_value: 0,
    activity_unit: 'kWh',
    activity_source: 'Invoice',
    emission_factor_value: 0,
    emission_factor_unit: 'kgCO2e/kWh',
    emission_factor_source: 'Supplier-specific EF',
    instrument_type: 'none',  // REC, PPA, GG, none
    uncertainty_pct: 5,
    notes: '',
  })

  const scope2Categories = [
    'Purchased electricity — grid',
    'Purchased electricity — renewable (REC/PPA)',
    'Purchased steam',
    'Purchased heat',
    'Purchased cooling',
    'District energy',
    'Other purchased energy',
  ]

  const instrumentTypes = [
    { value: 'none',  label: 'No instrument (residual mix)' },
    { value: 'REC',   label: 'REC — Renewable Energy Certificate' },
    { value: 'PPA',   label: 'PPA — Power Purchase Agreement' },
    { value: 'GG',    label: 'GG — Green Gas/Guarantee of Origin' },
    { value: 'other', label: 'Other contractual instrument' },
  ]

  onMount(async () => {
    const period = $activePeriod
    const org = $activeOrg
    if (!period || !org) return
    const all = await listSources(period.id, 2)
    locationSources = all.filter(s => s.scope2_method === 'location_based')
    marketSources   = all.filter(s => s.scope2_method === 'market_based')
    entities = (await listEntities(org.id)).map(e => ({ id: e.id, name: e.name }))
    if (entities.length > 0) {
      loc.entity_id = entities[0].id
      mkt.entity_id = entities[0].id
    }
  })

  async function reload() {
    const period = $activePeriod
    if (!period) return
    const all = await listSources(period.id, 2)
    locationSources = all.filter(s => s.scope2_method === 'location_based')
    marketSources   = all.filter(s => s.scope2_method === 'market_based')
  }

  async function addLocation() {
    const period = $activePeriod
    if (!period) return
    error = ''
    try {
      await createSource({
        entity_id: loc.entity_id,
        period_id: period.id,
        scope: 2,
        scope2_method: 'location_based',
        scope3_category: null,
        category_name: loc.category_name,
        ghg_type: 'CO2',
        activity_value: loc.activity_value,
        activity_unit: loc.activity_unit,
        activity_source: loc.activity_source,
        emission_factor_value: loc.emission_factor_value,
        emission_factor_unit: loc.emission_factor_unit,
        emission_factor_source: loc.emission_factor_source,
        emission_factor_citation: null,
        gwp_value: 1,
        biogenic_co2_tco2e: null,
        uncertainty_pct: loc.uncertainty_pct,
        notes: loc.notes || null,
      })
      await reload()
      showForm = null
    } catch (e) { error = String(e) }
  }

  async function addMarket() {
    const period = $activePeriod
    if (!period) return
    error = ''
    try {
      await createSource({
        entity_id: mkt.entity_id,
        period_id: period.id,
        scope: 2,
        scope2_method: 'market_based',
        scope3_category: null,
        category_name: mkt.category_name,
        ghg_type: 'CO2',
        activity_value: mkt.activity_value,
        activity_unit: mkt.activity_unit,
        activity_source: mkt.activity_source,
        emission_factor_value: mkt.emission_factor_value,
        emission_factor_unit: mkt.emission_factor_unit,
        emission_factor_source: mkt.emission_factor_source,
        emission_factor_citation: mkt.instrument_type !== 'none' ? `Instrument: ${mkt.instrument_type}` : null,
        gwp_value: 1,
        biogenic_co2_tco2e: null,
        uncertainty_pct: mkt.uncertainty_pct,
        notes: mkt.notes || null,
      })
      await reload()
      showForm = null
    } catch (e) { error = String(e) }
  }

  async function remove(id: number) {
    const period = $activePeriod
    if (!period) return
    await deleteSource(id, 'User deleted')
    await reload()
  }

  function fmt(n: number | null | undefined) {
    return n != null ? n.toFixed(3) : '—'
  }

  function totalTco2e(sources: EmissionSource[]) {
    return sources.reduce((sum, s) => sum + (s.emissions_tco2e ?? 0), 0)
  }
</script>

<div class="p-8">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-100">Scope 2 — Energy Indirect Emissions</h1>
    <p class="text-xs text-gray-500">GRI 305-2 · ISO 14064-1 §5.3.2 · Both methods are mandatory</p>
  </div>

  <!-- GRI 305-2 compliance notice -->
  <div class="mb-6 rounded-xl border border-blue-800/50 bg-blue-950/20 p-4">
    <p class="text-xs font-semibold text-blue-400">GRI 305-2 Dual-Method Requirement</p>
    <p class="mt-1 text-xs text-blue-300/70">
      GRI 305-2 and the GHG Protocol require reporting BOTH location-based AND market-based figures.
      Location-based uses grid-average emission factors. Market-based uses contractual instruments (RECs, PPAs) —
      zero if no instruments held, residual mix factor otherwise.
    </p>
  </div>

  <!-- Summary totals -->
  <div class="mb-6 grid gap-4 sm:grid-cols-2">
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
      <div class="mb-2 flex items-center justify-between">
        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Location-based</p>
        <button onclick={() => showForm = showForm === 'location' ? null : 'location'}
          class="rounded-lg bg-green-600 px-3 py-1 text-xs font-semibold text-white hover:bg-green-700">
          + Add
        </button>
      </div>
      <p class="text-2xl font-bold text-gray-100">{fmt(totalTco2e(locationSources))}</p>
      <p class="text-xs text-gray-500">tCO₂e · grid-average EF · {locationSources.length} source{locationSources.length !== 1 ? 's' : ''}</p>
    </div>
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
      <div class="mb-2 flex items-center justify-between">
        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Market-based</p>
        <button onclick={() => showForm = showForm === 'market' ? null : 'market'}
          class="rounded-lg bg-green-600 px-3 py-1 text-xs font-semibold text-white hover:bg-green-700">
          + Add
        </button>
      </div>
      <p class="text-2xl font-bold text-gray-100">{fmt(totalTco2e(marketSources))}</p>
      <p class="text-xs text-gray-500">tCO₂e · contractual instruments · {marketSources.length} source{marketSources.length !== 1 ? 's' : ''}</p>
    </div>
  </div>

  <!-- Location-based form -->
  {#if showForm === 'location'}
    <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
      <h3 class="mb-4 text-sm font-semibold text-gray-200">Add Location-based Source</h3>
      <p class="mb-4 text-xs text-gray-500">Use the grid-average emission factor for the region/country where energy is consumed.</p>
      <div class="grid gap-3 sm:grid-cols-2">
        <div>
          <label class="label">Category</label>
          <select bind:value={loc.category_name} class="input">
            <option value="">Select category…</option>
            {#each scope2Categories as cat}
              <option>{cat}</option>
            {/each}
          </select>
        </div>
        <div>
          <label class="label">Entity</label>
          <select bind:value={loc.entity_id} class="input">
            {#each entities as e}<option value={e.id}>{e.name}</option>{/each}
          </select>
        </div>
        <div>
          <label class="label">Activity value (kWh or MWh)</label>
          <input bind:value={loc.activity_value} type="number" step="0.001" class="input" />
        </div>
        <div>
          <label class="label">Activity unit</label>
          <input bind:value={loc.activity_unit} type="text" placeholder="kWh, MWh…" class="input" />
        </div>
        <div>
          <label class="label">Grid emission factor (kgCO₂e/kWh)</label>
          <input bind:value={loc.emission_factor_value} type="number" step="0.0001" class="input" />
        </div>
        <div>
          <label class="label">EF source (e.g. IEA, EPA eGRID, DEFRA)</label>
          <input bind:value={loc.emission_factor_source} type="text" class="input" />
        </div>
        <div>
          <label class="label">Data source</label>
          <select bind:value={loc.activity_source} class="input">
            <option>Meter</option><option>Invoice</option>
            <option>Utility Report</option><option>Estimate</option>
          </select>
        </div>
        <div>
          <label class="label">Uncertainty (%)</label>
          <input bind:value={loc.uncertainty_pct} type="number" step="1" min="0" max="100" class="input" />
        </div>
        <div class="sm:col-span-2">
          <label class="label">Notes</label>
          <input bind:value={loc.notes} type="text" class="input" />
        </div>
      </div>
      {#if error}<p class="mt-2 text-xs text-red-400">{error}</p>{/if}
      <div class="mt-4 flex gap-2">
        <button onclick={addLocation} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
        <button onclick={() => showForm = null} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">Cancel</button>
      </div>
    </div>
  {/if}

  <!-- Market-based form -->
  {#if showForm === 'market'}
    <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
      <h3 class="mb-4 text-sm font-semibold text-gray-200">Add Market-based Source</h3>
      <p class="mb-4 text-xs text-gray-500">Use zero if holding valid RECs/PPAs covering this consumption. Use residual mix EF if no instrument covers this source.</p>
      <div class="grid gap-3 sm:grid-cols-2">
        <div>
          <label class="label">Category</label>
          <select bind:value={mkt.category_name} class="input">
            <option value="">Select category…</option>
            {#each scope2Categories as cat}
              <option>{cat}</option>
            {/each}
          </select>
        </div>
        <div>
          <label class="label">Entity</label>
          <select bind:value={mkt.entity_id} class="input">
            {#each entities as e}<option value={e.id}>{e.name}</option>{/each}
          </select>
        </div>
        <div>
          <label class="label">Contractual instrument</label>
          <select bind:value={mkt.instrument_type} class="input">
            {#each instrumentTypes as it}<option value={it.value}>{it.label}</option>{/each}
          </select>
        </div>
        <div>
          <label class="label">Activity value</label>
          <input bind:value={mkt.activity_value} type="number" step="0.001" class="input" />
        </div>
        <div>
          <label class="label">Activity unit</label>
          <input bind:value={mkt.activity_unit} type="text" placeholder="kWh, MWh…" class="input" />
        </div>
        <div>
          <label class="label">
            Emission factor
            {#if mkt.instrument_type === 'REC' || mkt.instrument_type === 'PPA' || mkt.instrument_type === 'GG'}
              <span class="ml-1 text-green-600">(0 if fully covered)</span>
            {/if}
          </label>
          <input bind:value={mkt.emission_factor_value} type="number" step="0.0001" min="0" class="input" />
        </div>
        <div>
          <label class="label">EF unit</label>
          <input bind:value={mkt.emission_factor_unit} type="text" class="input" />
        </div>
        <div>
          <label class="label">EF source</label>
          <input bind:value={mkt.emission_factor_source} type="text" class="input" />
        </div>
        <div>
          <label class="label">Uncertainty (%)</label>
          <input bind:value={mkt.uncertainty_pct} type="number" step="1" min="0" max="100" class="input" />
        </div>
        <div class="sm:col-span-2">
          <label class="label">Notes (instrument registry, vintage year, etc.)</label>
          <input bind:value={mkt.notes} type="text" class="input" />
        </div>
      </div>
      {#if error}<p class="mt-2 text-xs text-red-400">{error}</p>{/if}
      <div class="mt-4 flex gap-2">
        <button onclick={addMarket} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
        <button onclick={() => showForm = null} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">Cancel</button>
      </div>
    </div>
  {/if}

  <!-- Location-based table -->
  <div class="mb-6">
    <h2 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">Location-based Sources</h2>
    <div class="overflow-hidden rounded-xl border border-gray-800">
      <table class="w-full text-sm">
        <thead class="border-b border-gray-800 bg-gray-900/60">
          <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
            <th class="px-4 py-3">Category</th>
            <th class="px-4 py-3">Activity</th>
            <th class="px-4 py-3">Grid EF</th>
            <th class="px-4 py-3">tCO₂e</th>
            <th class="px-4 py-3">±%</th>
            <th class="px-4 py-3"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800 bg-gray-900">
          {#each locationSources as s}
            <tr class="hover:bg-gray-800/40">
              <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
              <td class="px-4 py-3 text-gray-300">{s.activity_value} {s.activity_unit}</td>
              <td class="px-4 py-3 text-xs text-gray-500">{s.emission_factor_value} {s.emission_factor_unit}</td>
              <td class="px-4 py-3 font-semibold text-gray-200">{fmt(s.emissions_tco2e)}</td>
              <td class="px-4 py-3 text-xs text-gray-500">{s.uncertainty_pct != null ? `±${s.uncertainty_pct}%` : '—'}</td>
              <td class="px-4 py-3">
                <button onclick={() => remove(s.id)} class="text-xs text-red-500 hover:text-red-400">✕</button>
              </td>
            </tr>
          {:else}
            <tr><td colspan="6" class="px-4 py-6 text-center text-xs text-gray-500">No location-based sources. Required for GRI 305-2.</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Market-based table -->
  <div>
    <h2 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">Market-based Sources</h2>
    <div class="overflow-hidden rounded-xl border border-gray-800">
      <table class="w-full text-sm">
        <thead class="border-b border-gray-800 bg-gray-900/60">
          <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
            <th class="px-4 py-3">Category</th>
            <th class="px-4 py-3">Activity</th>
            <th class="px-4 py-3">EF (contractual)</th>
            <th class="px-4 py-3">Instrument</th>
            <th class="px-4 py-3">tCO₂e</th>
            <th class="px-4 py-3">±%</th>
            <th class="px-4 py-3"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800 bg-gray-900">
          {#each marketSources as s}
            <tr class="hover:bg-gray-800/40">
              <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
              <td class="px-4 py-3 text-gray-300">{s.activity_value} {s.activity_unit}</td>
              <td class="px-4 py-3 text-xs text-gray-500">{s.emission_factor_value} {s.emission_factor_unit}</td>
              <td class="px-4 py-3 text-xs text-gray-400">{s.emission_factor_citation ?? '—'}</td>
              <td class="px-4 py-3 font-semibold text-gray-200">{fmt(s.emissions_tco2e)}</td>
              <td class="px-4 py-3 text-xs text-gray-500">{s.uncertainty_pct != null ? `±${s.uncertainty_pct}%` : '—'}</td>
              <td class="px-4 py-3">
                <button onclick={() => remove(s.id)} class="text-xs text-red-500 hover:text-red-400">✕</button>
              </td>
            </tr>
          {:else}
            <tr><td colspan="7" class="px-4 py-6 text-center text-xs text-gray-500">No market-based sources. Required for GRI 305-2.</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .label { @apply mb-1 block text-xs font-medium text-gray-400; }
  .input { @apply w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none; }
</style>
