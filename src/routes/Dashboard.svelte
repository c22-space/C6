<script lang="ts">
  import { onMount } from 'svelte'
  import { activeOrg, activePeriod } from '$lib/stores/app'
  import { calculatePeriod } from '$lib/tauri'
  import type { PeriodInventory } from '$lib/tauri'

  let inventory = $state<PeriodInventory | null>(null)
  let loading = $state(false)
  let error = $state('')

  onMount(async () => {
    const period = $activePeriod
    if (!period) return
    loading = true
    try {
      inventory = await calculatePeriod(period.id)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  })

  function fmt(n: number | undefined | null) {
    if (n == null) return '—'
    return n.toFixed(2)
  }
</script>

<div class="p-8">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-100">Dashboard</h1>
    {#if $activePeriod}
      <p class="text-sm text-gray-500">
        {$activeOrg?.name} · {$activePeriod.year} · {$activePeriod.gwp_ar_version} GWP values
      </p>
    {/if}
  </div>

  {#if loading}
    <p class="text-sm text-gray-500">Calculating…</p>
  {:else if error}
    <div class="rounded-xl border border-red-800 bg-red-950/20 p-4 text-sm text-red-400">{error}</div>
  {:else if inventory}
    <!-- Scope summary cards -->
    <div class="mb-6 grid gap-4 sm:grid-cols-3">
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">Scope 1</p>
        <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope1.gross_tco2e)}</p>
        <p class="text-xs text-gray-500">tCO₂e · direct emissions</p>
        {#if inventory.scope1.biogenic_co2_tco2e > 0}
          <p class="mt-1 text-xs text-gray-600">+ {fmt(inventory.scope1.biogenic_co2_tco2e)} tCO₂ biogenic (separate)</p>
        {/if}
      </div>

      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">Scope 2</p>
        <div class="space-y-1">
          <div class="flex items-baseline gap-1.5">
            <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope2.location_based_tco2e)}</p>
            <span class="text-xs text-gray-600">location</span>
          </div>
          <div class="flex items-baseline gap-1.5">
            <p class="text-lg font-semibold text-gray-400">{fmt(inventory.scope2.market_based_tco2e)}</p>
            <span class="text-xs text-gray-600">market</span>
          </div>
        </div>
        <p class="mt-1 text-xs text-gray-500">tCO₂e · energy indirect</p>
      </div>

      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">Scope 3</p>
        <p class="text-2xl font-bold text-gray-100">{fmt(inventory.scope3.gross_tco2e)}</p>
        <p class="text-xs text-gray-500">tCO₂e · other indirect</p>
        <p class="mt-1 text-xs text-gray-600">
          {inventory.scope3.excluded_categories.length} categor{inventory.scope3.excluded_categories.length === 1 ? 'y' : 'ies'} excluded
        </p>
      </div>
    </div>

    <!-- Total -->
    <div class="mb-6 rounded-xl border border-green-900/50 bg-green-950/20 p-5">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-semibold uppercase tracking-wider text-green-600">Total (Scope 1+2+3)</p>
          <p class="text-3xl font-black text-green-400">{fmt(inventory.total_tco2e)} <span class="text-base font-normal text-green-700">tCO₂e</span></p>
        </div>
        <div class="text-right">
          <p class="text-xs text-gray-600">Scope 1+2 only</p>
          <p class="text-xl font-bold text-gray-400">{fmt(inventory.scope1_scope2_tco2e)} tCO₂e</p>
        </div>
      </div>
    </div>

    <!-- Scope 3 category breakdown -->
    {#if inventory.scope3.categories.some(c => c.total_tco2e > 0)}
      <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
        <h2 class="mb-4 text-xs font-semibold uppercase tracking-wider text-gray-500">Scope 3 Categories</h2>
        <div class="space-y-1.5">
          {#each inventory.scope3.categories.filter(c => c.total_tco2e > 0) as cat}
            <div class="flex items-center justify-between py-1">
              <div class="flex items-center gap-2">
                <span class="w-5 text-right text-xs text-gray-600">{cat.category}</span>
                <span class="text-sm text-gray-300">{cat.category_name}</span>
                <span class="text-xs text-gray-600">{cat.direction}</span>
              </div>
              <span class="text-sm font-semibold text-gray-200">{fmt(cat.total_tco2e)} tCO₂e</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
      <p class="text-sm text-gray-500">No data yet. Add emission sources to get started.</p>
    </div>
  {/if}
</div>
