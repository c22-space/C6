<script lang="ts">
  import { onMount } from 'svelte'
  import { activePeriod, activeOrg } from '$lib/stores/app'
  import { listSources, createSource, deleteSource, listEmissionFactors, listEntities } from '$lib/tauri'
  import type { EmissionSource } from '$lib/tauri'

  let sources = $state<EmissionSource[]>([])
  let factors = $state<Record<string, unknown>[]>([])
  let entities = $state<{ id: number; name: string }[]>([])
  let showForm = $state(false)
  let error = $state('')

  // Form state
  let f = $state({
    entity_id: 0,
    category_name: '',
    ghg_type: 'CO2',
    activity_value: 0,
    activity_unit: 'kWh',
    activity_source: 'Invoice',
    emission_factor_value: 0,
    emission_factor_unit: 'kgCO2e/kWh',
    emission_factor_source: 'DEFRA 2024',
    gwp_value: 1,
    uncertainty_pct: 5,
    notes: '',
  })

  const ghgTypes = ['CO2','CH4_non_fossil','CH4_fossil','N2O','HFC','PFC','SF6','NF3','other']

  const scope1Categories = [
    'Stationary combustion — natural gas',
    'Stationary combustion — diesel',
    'Stationary combustion — LPG',
    'Stationary combustion — coal',
    'Mobile combustion — company vehicles (petrol)',
    'Mobile combustion — company vehicles (diesel)',
    'Fugitive emissions — refrigerants',
    'Fugitive emissions — methane (natural gas)',
    'Process emissions',
    'Other direct emissions',
  ]

  onMount(async () => {
    const period = $activePeriod
    const org = $activeOrg
    if (!period || !org) return
    sources = await listSources(period.id, 1)
    factors = await listEmissionFactors('fuel', undefined) as Record<string, unknown>[]
    entities = (await listEntities(org.id)).map(e => ({ id: e.id, name: e.name }))
    if (entities.length > 0) f.entity_id = entities[0].id
  })

  async function addSource() {
    const period = $activePeriod
    if (!period) return
    error = ''
    try {
      await createSource({
        entity_id: f.entity_id,
        period_id: period.id,
        scope: 1,
        scope2_method: null,
        scope3_category: null,
        category_name: f.category_name,
        ghg_type: f.ghg_type,
        activity_value: f.activity_value,
        activity_unit: f.activity_unit,
        activity_source: f.activity_source,
        emission_factor_value: f.emission_factor_value,
        emission_factor_unit: f.emission_factor_unit,
        emission_factor_source: f.emission_factor_source,
        emission_factor_citation: null,
        gwp_value: f.gwp_value,
        biogenic_co2_tco2e: null,
        uncertainty_pct: f.uncertainty_pct,
        notes: f.notes || null,
      })
      sources = await listSources(period.id, 1)
      showForm = false
    } catch (e) {
      error = String(e)
    }
  }

  async function remove(id: number) {
    const period = $activePeriod
    if (!period) return
    await deleteSource(id, 'User deleted')
    sources = await listSources(period.id, 1)
  }

  function fmt(n: number | null | undefined) {
    return n != null ? n.toFixed(3) : '—'
  }
</script>

<div class="p-8">
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h1 class="text-xl font-bold text-gray-100">Scope 1 — Direct Emissions</h1>
      <p class="text-xs text-gray-500">GRI 305-1 · ISO 14064-1 §5.3.1 · Owned/controlled sources</p>
    </div>
    <button onclick={() => showForm = !showForm}
      class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">
      + Add source
    </button>
  </div>

  {#if showForm}
    <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
      <h3 class="mb-4 text-sm font-semibold text-gray-200">Add Scope 1 Source</h3>
      <div class="grid gap-3 sm:grid-cols-2">
        <div>
          <label class="label">Category</label>
          <select bind:value={f.category_name} class="input">
            <option value="">Select category…</option>
            {#each scope1Categories as cat}
              <option>{cat}</option>
            {/each}
          </select>
        </div>
        <div>
          <label class="label">GHG Type</label>
          <select bind:value={f.ghg_type} class="input">
            {#each ghgTypes as g}<option>{g}</option>{/each}
          </select>
        </div>
        <div>
          <label class="label">Activity value</label>
          <input bind:value={f.activity_value} type="number" step="0.001" class="input" />
        </div>
        <div>
          <label class="label">Activity unit</label>
          <input bind:value={f.activity_unit} type="text" placeholder="kWh, L, m3, kg…" class="input" />
        </div>
        <div>
          <label class="label">Emission factor (kgCO₂e/unit)</label>
          <input bind:value={f.emission_factor_value} type="number" step="0.0001" class="input" />
        </div>
        <div>
          <label class="label">EF source</label>
          <input bind:value={f.emission_factor_source} type="text" class="input" />
        </div>
        <div>
          <label class="label">GWP value</label>
          <input bind:value={f.gwp_value} type="number" step="0.1" class="input" />
        </div>
        <div>
          <label class="label">Uncertainty (%)</label>
          <input bind:value={f.uncertainty_pct} type="number" step="1" min="0" max="100" class="input" />
        </div>
        <div class="sm:col-span-2">
          <label class="label">Notes</label>
          <input bind:value={f.notes} type="text" class="input" />
        </div>
      </div>
      {#if error}
        <p class="mt-2 text-xs text-red-400">{error}</p>
      {/if}
      <div class="mt-4 flex gap-2">
        <button onclick={addSource} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">
          Save
        </button>
        <button onclick={() => showForm = false} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">
          Cancel
        </button>
      </div>
    </div>
  {/if}

  <!-- Sources table -->
  <div class="overflow-hidden rounded-xl border border-gray-800">
    <table class="w-full text-sm">
      <thead class="border-b border-gray-800 bg-gray-900/60">
        <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
          <th class="px-4 py-3">Category</th>
          <th class="px-4 py-3">GHG</th>
          <th class="px-4 py-3">Activity</th>
          <th class="px-4 py-3">EF</th>
          <th class="px-4 py-3">GWP</th>
          <th class="px-4 py-3">tCO₂e</th>
          <th class="px-4 py-3">±%</th>
          <th class="px-4 py-3"></th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800 bg-gray-900">
        {#each sources as s}
          <tr class="hover:bg-gray-800/40">
            <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
            <td class="px-4 py-3 font-mono text-xs text-gray-400">{s.ghg_type}</td>
            <td class="px-4 py-3 text-gray-300">{s.activity_value} {s.activity_unit}</td>
            <td class="px-4 py-3 text-xs text-gray-500">{s.emission_factor_value} {s.emission_factor_unit}</td>
            <td class="px-4 py-3 text-xs text-gray-500">{s.gwp_value}</td>
            <td class="px-4 py-3 font-semibold text-gray-200">{fmt(s.emissions_tco2e)}</td>
            <td class="px-4 py-3 text-xs text-gray-500">{s.uncertainty_pct != null ? `±${s.uncertainty_pct}%` : '—'}</td>
            <td class="px-4 py-3">
              <button onclick={() => remove(s.id)} class="text-xs text-red-500 hover:text-red-400">✕</button>
            </td>
          </tr>
        {:else}
          <tr><td colspan="8" class="px-4 py-8 text-center text-sm text-gray-500">No Scope 1 sources yet.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .label { @apply mb-1 block text-xs font-medium text-gray-400; }
  .input { @apply w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none; }
</style>
