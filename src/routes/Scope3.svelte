<script lang="ts">
  import { onMount } from 'svelte'
  import { activePeriod, activeOrg } from '$lib/stores/app'
  import { listSources, createSource, deleteSource, listEntities } from '$lib/tauri'
  import type { EmissionSource } from '$lib/tauri'

  // GHG Protocol Corporate Value Chain — all 15 categories
  const CATEGORIES = [
    { num: 1,  name: 'Purchased goods and services',         dir: 'upstream',   hint: 'Cradle-to-gate emissions from goods/services purchased in the reporting year' },
    { num: 2,  name: 'Capital goods',                        dir: 'upstream',   hint: 'Extraction, production, and transport of capital goods purchased/acquired' },
    { num: 3,  name: 'Fuel- and energy-related activities',  dir: 'upstream',   hint: 'Upstream emissions from fuel/energy not in Scope 1 or 2 (extraction, refining, T&D losses)' },
    { num: 4,  name: 'Upstream transportation and distribution', dir: 'upstream', hint: 'Transport of purchased products between suppliers and company' },
    { num: 5,  name: 'Waste generated in operations',        dir: 'upstream',   hint: 'Disposal and treatment of waste generated in the reporting year' },
    { num: 6,  name: 'Business travel',                      dir: 'upstream',   hint: 'Transportation by employees for business in vehicles not owned/controlled by company' },
    { num: 7,  name: 'Employee commuting',                   dir: 'upstream',   hint: 'Transportation of employees between home and work in non-company vehicles' },
    { num: 8,  name: 'Upstream leased assets',               dir: 'upstream',   hint: 'Operation of assets leased by the reporting company (not in Scope 1/2)' },
    { num: 9,  name: 'Downstream transportation and distribution', dir: 'downstream', hint: 'Transport of sold products after point of sale' },
    { num: 10, name: 'Processing of sold products',          dir: 'downstream', hint: 'Processing of intermediate products sold by company' },
    { num: 11, name: 'Use of sold products',                 dir: 'downstream', hint: 'End-use of goods and services sold in the reporting year' },
    { num: 12, name: 'End-of-life treatment of sold products', dir: 'downstream', hint: 'Waste disposal and treatment of sold products at end of life' },
    { num: 13, name: 'Downstream leased assets',             dir: 'downstream', hint: 'Operation of assets owned by the company and leased to others' },
    { num: 14, name: 'Franchises',                           dir: 'downstream', hint: 'Operation of franchises not included in Scope 1/2' },
    { num: 15, name: 'Investments',                          dir: 'downstream', hint: 'Investment activities (equity, debt, project finance) for finance/insurance sectors' },
  ]

  const ghgTypes = ['CO2','CH4_non_fossil','CH4_fossil','N2O','HFC','PFC','SF6','NF3','other']

  type CategorySources = Record<number, EmissionSource[]>

  let byCategory = $state<CategorySources>({})
  let entities = $state<{ id: number; name: string }[]>([])
  let openForm = $state<number | null>(null)   // category num
  let openExclude = $state<number | null>(null)
  let excludeReasons = $state<Record<number, string>>({})
  let error = $state('')

  let f = $state({
    entity_id: 0,
    activity_value: 0,
    activity_unit: 't',
    activity_source: 'Supplier Report',
    ghg_type: 'CO2',
    emission_factor_value: 0,
    emission_factor_unit: 'kgCO2e/t',
    emission_factor_source: 'GHG Protocol',
    gwp_value: 1,
    uncertainty_pct: 15,
    notes: '',
  })

  onMount(async () => {
    const period = $activePeriod
    const org = $activeOrg
    if (!period || !org) return
    const all = await listSources(period.id, 3)
    const grouped: CategorySources = {}
    for (let i = 1; i <= 15; i++) grouped[i] = []
    for (const s of all) {
      if (s.scope3_category != null) {
        grouped[s.scope3_category] = [...(grouped[s.scope3_category] ?? []), s]
      }
    }
    byCategory = grouped
    entities = (await listEntities(org.id)).map(e => ({ id: e.id, name: e.name }))
    if (entities.length > 0) f.entity_id = entities[0].id
  })

  async function reload() {
    const period = $activePeriod
    if (!period) return
    const all = await listSources(period.id, 3)
    const grouped: CategorySources = {}
    for (let i = 1; i <= 15; i++) grouped[i] = []
    for (const s of all) {
      if (s.scope3_category != null) {
        grouped[s.scope3_category] = [...(grouped[s.scope3_category] ?? []), s]
      }
    }
    byCategory = grouped
  }

  async function addSource(cat: typeof CATEGORIES[0]) {
    const period = $activePeriod
    if (!period) return
    error = ''
    try {
      await createSource({
        entity_id: f.entity_id,
        period_id: period.id,
        scope: 3,
        scope2_method: null,
        scope3_category: cat.num,
        category_name: cat.name,
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
      await reload()
      openForm = null
    } catch (e) { error = String(e) }
  }

  async function markExcluded(cat: typeof CATEGORIES[0]) {
    const period = $activePeriod
    const reason = excludeReasons[cat.num]?.trim()
    if (!period || !reason) { error = 'Exclusion reason is required (ISO 14064-1)'; return }
    error = ''
    try {
      // Insert a placeholder source flagged as excluded
      await createSource({
        entity_id: entities[0]?.id ?? 0,
        period_id: period.id,
        scope: 3,
        scope2_method: null,
        scope3_category: cat.num,
        category_name: cat.name,
        ghg_type: 'CO2',
        activity_value: 0,
        activity_unit: 'n/a',
        activity_source: 'Excluded',
        emission_factor_value: 0,
        emission_factor_unit: 'n/a',
        emission_factor_source: 'n/a',
        emission_factor_citation: null,
        gwp_value: 1,
        biogenic_co2_tco2e: null,
        uncertainty_pct: null,
        notes: `EXCLUDED: ${reason}`,
      })
      await reload()
      openExclude = null
    } catch (e) { error = String(e) }
  }

  async function remove(id: number) {
    const period = $activePeriod
    if (!period) return
    await deleteSource(id, 'User deleted')
    await reload()
  }

  function catTotal(cat: number): number {
    return (byCategory[cat] ?? []).reduce((s, r) => s + (r.emissions_tco2e ?? 0), 0)
  }

  function fmt(n: number) { return n.toFixed(3) }

  function isExcluded(cat: number): boolean {
    return (byCategory[cat] ?? []).some(s => s.notes?.startsWith('EXCLUDED:'))
  }
</script>

<div class="p-8">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-100">Scope 3 — Other Indirect Emissions</h1>
    <p class="text-xs text-gray-500">GRI 305-3 · ISO 14064-1 §5.3.3 · GHG Protocol Corporate Value Chain — 15 categories</p>
  </div>

  <div class="mb-4 rounded-xl border border-yellow-800/40 bg-yellow-950/15 p-3">
    <p class="text-xs text-yellow-400">
      <span class="font-semibold">ISO 14064-1 requirement:</span> All excluded categories must have a documented reason.
      Exclusion without a reason is not compliant.
    </p>
  </div>

  {#if error}<p class="mb-4 text-xs text-red-400">{error}</p>{/if}

  <!-- Category rows -->
  <div class="space-y-2">
    {#each CATEGORIES as cat}
      {@const sources = byCategory[cat.num] ?? []}
      {@const total = catTotal(cat.num)}
      {@const excluded = isExcluded(cat.num)}
      <div class="overflow-hidden rounded-xl border {excluded ? 'border-gray-800/50 opacity-60' : 'border-gray-800'} bg-gray-900">
        <!-- Category header -->
        <div class="flex items-center justify-between px-4 py-3">
          <div class="flex items-center gap-3">
            <span class="flex h-6 w-6 items-center justify-center rounded-full bg-gray-800 text-xs font-bold text-gray-400">{cat.num}</span>
            <div>
              <p class="text-sm font-medium text-gray-200">{cat.name}</p>
              <p class="text-xs text-gray-600">{cat.dir} · {cat.hint}</p>
            </div>
          </div>
          <div class="flex items-center gap-3">
            {#if total > 0}
              <span class="text-sm font-semibold text-gray-200">{fmt(total)} tCO₂e</span>
            {:else if excluded}
              <span class="text-xs text-gray-600 italic">excluded</span>
            {:else}
              <span class="text-xs text-gray-600">not entered</span>
            {/if}
            {#if !excluded}
              <button onclick={() => openForm = openForm === cat.num ? null : cat.num}
                class="rounded-lg bg-green-700/60 px-3 py-1 text-xs font-medium text-green-300 hover:bg-green-700">
                + Add
              </button>
              <button onclick={() => openExclude = openExclude === cat.num ? null : cat.num}
                class="rounded-lg border border-gray-700 px-3 py-1 text-xs text-gray-500 hover:border-gray-600 hover:text-gray-400">
                Exclude
              </button>
            {/if}
          </div>
        </div>

        <!-- Existing sources mini-table -->
        {#if sources.length > 0 && !excluded}
          <div class="border-t border-gray-800/50 px-4 pb-2">
            <table class="w-full text-xs">
              <thead>
                <tr class="text-gray-600">
                  <th class="py-1 text-left font-medium">Activity</th>
                  <th class="py-1 text-left font-medium">EF</th>
                  <th class="py-1 text-left font-medium">GHG</th>
                  <th class="py-1 text-left font-medium">tCO₂e</th>
                  <th class="py-1"></th>
                </tr>
              </thead>
              <tbody>
                {#each sources as s}
                  <tr class="text-gray-400">
                    <td class="py-0.5">{s.activity_value} {s.activity_unit}</td>
                    <td class="py-0.5">{s.emission_factor_value} {s.emission_factor_unit}</td>
                    <td class="py-0.5 font-mono">{s.ghg_type}</td>
                    <td class="py-0.5 font-semibold text-gray-300">{s.emissions_tco2e?.toFixed(3) ?? '—'}</td>
                    <td class="py-0.5 text-right">
                      <button onclick={() => remove(s.id)} class="text-red-600 hover:text-red-400">✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}

        <!-- Add form (inline) -->
        {#if openForm === cat.num}
          <div class="border-t border-gray-700 bg-gray-900/60 p-4">
            <div class="grid gap-3 sm:grid-cols-3">
              <div>
                <label class="label">Activity value</label>
                <input bind:value={f.activity_value} type="number" step="0.001" class="input" />
              </div>
              <div>
                <label class="label">Activity unit</label>
                <input bind:value={f.activity_unit} type="text" class="input" />
              </div>
              <div>
                <label class="label">Data source</label>
                <select bind:value={f.activity_source} class="input">
                  <option>Supplier Report</option><option>Invoice</option>
                  <option>Spend-based</option><option>Estimate</option><option>Average data</option>
                </select>
              </div>
              <div>
                <label class="label">GHG type</label>
                <select bind:value={f.ghg_type} class="input">
                  {#each ghgTypes as g}<option>{g}</option>{/each}
                </select>
              </div>
              <div>
                <label class="label">Emission factor</label>
                <input bind:value={f.emission_factor_value} type="number" step="0.0001" class="input" />
              </div>
              <div>
                <label class="label">EF unit</label>
                <input bind:value={f.emission_factor_unit} type="text" class="input" />
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
              <div class="sm:col-span-3">
                <label class="label">Notes</label>
                <input bind:value={f.notes} type="text" class="input" />
              </div>
            </div>
            <div class="mt-3 flex gap-2">
              <button onclick={() => addSource(cat)} class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">Save</button>
              <button onclick={() => openForm = null} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">Cancel</button>
            </div>
          </div>
        {/if}

        <!-- Exclude form (inline) -->
        {#if openExclude === cat.num}
          <div class="border-t border-gray-700 bg-gray-900/60 p-4">
            <p class="mb-2 text-xs text-yellow-500">ISO 14064-1 requires a documented reason for excluding any Scope 3 category.</p>
            <div class="flex gap-2">
              <input
                bind:value={excludeReasons[cat.num]}
                type="text"
                placeholder="e.g. Not material — category represents &lt;1% of total emissions"
                class="input flex-1"
              />
              <button onclick={() => markExcluded(cat)} class="rounded-lg border border-yellow-700 px-4 py-2 text-sm text-yellow-500 hover:border-yellow-600">
                Confirm
              </button>
              <button onclick={() => openExclude = null} class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-500 hover:border-gray-600">
                Cancel
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .label { @apply mb-1 block text-xs font-medium text-gray-400; }
  .input { @apply w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none; }
</style>
