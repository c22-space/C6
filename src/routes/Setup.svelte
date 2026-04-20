<script lang="ts">
  import { createOrg, createEntity, createPeriod } from '$lib/tauri'
  import { activeOrg, activePeriod, currentRoute } from '$lib/stores/app'

  let step = $state(1)
  let error = $state('')
  let loading = $state(false)

  // Step 1: Org
  let orgName = $state('')
  let boundaryMethod = $state('operational_control')
  let baseYear = $state(new Date().getFullYear() - 1)
  let currency = $state('USD')

  // Step 2: Reporting period
  let periodYear = $state(new Date().getFullYear() - 1)
  let gwpAr = $state('AR6')

  async function createOrgStep() {
    if (!orgName.trim()) { error = 'Organisation name is required'; return }
    loading = true; error = ''
    try {
      const org = await createOrg({
        name: orgName.trim(),
        boundary_method: boundaryMethod,
        base_year: baseYear,
        reporting_currency: currency,
      })
      activeOrg.set(org)

      // Create a default parent entity
      await createEntity({
        org_id: org.id,
        name: orgName.trim(),
        type: 'parent',
        is_financially_controlled: true,
        is_operationally_controlled: true,
      })

      step = 2
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function createPeriodStep() {
    const org = $activeOrg
    if (!org) return
    loading = true; error = ''
    try {
      const start = `${periodYear}-01-01`
      const end = `${periodYear}-12-31`
      const period = await createPeriod({
        org_id: org.id,
        year: periodYear,
        start_date: start,
        end_date: end,
        gwp_ar_version: gwpAr,
      })
      activePeriod.set(period)
      currentRoute.set('/dashboard')
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-950 p-6">
  <div class="w-full max-w-lg">
    <div class="mb-8 text-center">
      <div class="mb-2 text-3xl font-bold text-green-500">c12</div>
      <p class="text-sm text-gray-500">Carbon accounting for GRI 305, ISO 14064 & UNGC</p>
    </div>

    <!-- Step indicator -->
    <div class="mb-8 flex items-center gap-2">
      {#each [1, 2] as s}
        <div class="flex items-center gap-2">
          <div class="flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold
            {step === s ? 'bg-green-600 text-white' : step > s ? 'bg-green-900 text-green-400' : 'bg-gray-800 text-gray-500'}">
            {s}
          </div>
          <span class="text-xs {step === s ? 'text-gray-200' : 'text-gray-600'}">
            {s === 1 ? 'Organisation' : 'Reporting period'}
          </span>
        </div>
        {#if s < 2}
          <div class="flex-1 h-px bg-gray-800"></div>
        {/if}
      {/each}
    </div>

    <div class="rounded-2xl border border-gray-800 bg-gray-900 p-6">
      {#if step === 1}
        <h2 class="mb-5 text-base font-semibold text-gray-100">Set up your organisation</h2>

        <div class="space-y-4">
          <div>
            <label class="mb-1.5 block text-xs font-medium text-gray-400">Organisation name</label>
            <input bind:value={orgName} type="text" placeholder="Acme Corp"
              class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                     placeholder:text-gray-600 focus:border-green-600 focus:outline-none focus:ring-1 focus:ring-green-600" />
          </div>

          <div>
            <label class="mb-1.5 block text-xs font-medium text-gray-400">
              Organisational boundary method
              <span class="ml-1 text-gray-600">(ISO 14064-1 §5.2 — select one)</span>
            </label>
            <div class="space-y-2">
              {#each [
                { value: 'operational_control', label: 'Operational Control', desc: '100% of entities where you control day-to-day operations (most common)' },
                { value: 'financial_control',   label: 'Financial Control',   desc: '100% of entities where you control financial policies' },
                { value: 'equity_share',        label: 'Equity Share',        desc: 'Pro-rata share based on ownership percentage' },
              ] as opt}
                <label class="flex cursor-pointer items-start gap-3 rounded-lg border p-3 transition-colors
                  {boundaryMethod === opt.value ? 'border-green-700 bg-green-950/30' : 'border-gray-800 hover:border-gray-700'}">
                  <input type="radio" bind:group={boundaryMethod} value={opt.value} class="mt-0.5" />
                  <div>
                    <div class="text-sm font-medium text-gray-200">{opt.label}</div>
                    <div class="text-xs text-gray-500">{opt.desc}</div>
                  </div>
                </label>
              {/each}
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="mb-1.5 block text-xs font-medium text-gray-400">Base year</label>
              <input bind:value={baseYear} type="number" min="2000" max="2030"
                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                       focus:border-green-600 focus:outline-none" />
            </div>
            <div>
              <label class="mb-1.5 block text-xs font-medium text-gray-400">Currency</label>
              <select bind:value={currency}
                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                       focus:border-green-600 focus:outline-none">
                <option>USD</option><option>EUR</option><option>GBP</option>
                <option>AUD</option><option>CAD</option><option>JPY</option>
              </select>
            </div>
          </div>
        </div>

        {#if error}
          <p class="mt-3 text-xs text-red-400">{error}</p>
        {/if}

        <button onclick={createOrgStep} disabled={loading}
          class="mt-5 w-full rounded-lg bg-green-600 px-4 py-2.5 text-sm font-semibold text-white
                 hover:bg-green-700 disabled:opacity-50 transition-colors">
          {loading ? 'Creating…' : 'Continue →'}
        </button>

      {:else}
        <h2 class="mb-5 text-base font-semibold text-gray-100">Create your first reporting period</h2>

        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="mb-1.5 block text-xs font-medium text-gray-400">Reporting year</label>
              <input bind:value={periodYear} type="number" min="2000" max="2030"
                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                       focus:border-green-600 focus:outline-none" />
            </div>
            <div>
              <label class="mb-1.5 block text-xs font-medium text-gray-400">
                GWP values
                <span class="ml-1 text-gray-600">(IPCC AR)</span>
              </label>
              <select bind:value={gwpAr}
                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                       focus:border-green-600 focus:outline-none">
                <option value="AR6">AR6 (2021) — Recommended</option>
                <option value="AR5">AR5 (2013)</option>
                <option value="AR4">AR4 (2007)</option>
              </select>
            </div>
          </div>

          <div class="rounded-lg border border-yellow-800/50 bg-yellow-950/20 p-3">
            <p class="text-xs text-yellow-500">
              <span class="font-semibold">IPCC AR6 recommended.</span>
              AR6 values are current as of 2024 and required for new GRI 305 reports.
              Use AR4/AR5 only for historical comparisons.
            </p>
          </div>
        </div>

        {#if error}
          <p class="mt-3 text-xs text-red-400">{error}</p>
        {/if}

        <div class="mt-5 flex gap-3">
          <button onclick={() => step = 1}
            class="rounded-lg border border-gray-700 px-4 py-2.5 text-sm font-medium text-gray-400
                   hover:border-gray-600 hover:text-gray-200 transition-colors">
            ← Back
          </button>
          <button onclick={createPeriodStep} disabled={loading}
            class="flex-1 rounded-lg bg-green-600 px-4 py-2.5 text-sm font-semibold text-white
                   hover:bg-green-700 disabled:opacity-50 transition-colors">
            {loading ? 'Setting up…' : 'Start accounting →'}
          </button>
        </div>
      {/if}
    </div>

    <p class="mt-6 text-center text-[10px] text-gray-700">
      Built by <a href="https://c22.space" class="hover:text-gray-500">c22</a> ·
      <a href="https://c22.space/hire" class="hover:text-gray-500">Hire us →</a>
    </p>
  </div>
</div>
