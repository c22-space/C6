<script lang="ts">
  import { onMount } from 'svelte'
  import { currentRoute, activeOrg, activePeriod } from '$lib/stores/app'
  import { listOrgs, listPeriods } from '$lib/tauri'
  import Sidebar from '$lib/components/Sidebar.svelte'
  import Setup from './routes/Setup.svelte'
  import Dashboard from './routes/Dashboard.svelte'
  import Scope1 from './routes/Scope1.svelte'
  import Scope2 from './routes/Scope2.svelte'
  import Scope3 from './routes/Scope3.svelte'
  import Reports from './routes/Reports.svelte'
  import Ungc from './routes/Ungc.svelte'
  import Settings from './routes/Settings.svelte'

  let loading = true

  onMount(async () => {
    try {
      const orgs = await listOrgs()
      if (orgs.length > 0) {
        activeOrg.set(orgs[0])
        const periods = await listPeriods(orgs[0].id)
        if (periods.length > 0) activePeriod.set(periods[0])
        currentRoute.set('/dashboard')
      } else {
        currentRoute.set('/setup')
      }
    } catch (e) {
      console.error('Failed to initialise:', e)
      currentRoute.set('/setup')
    } finally {
      loading = false
    }
  })

  const routes: Record<string, typeof Dashboard> = {
    '/dashboard': Dashboard,
    '/sources/scope1': Scope1,
    '/sources/scope2': Scope2,
    '/sources/scope3': Scope3,
    '/reports': Reports,
    '/ungc': Ungc,
    '/settings': Settings,
  }
</script>

{#if loading}
  <div class="flex h-screen items-center justify-center bg-gray-950">
    <div class="text-center">
      <div class="mb-3 text-2xl font-bold text-green-500">c12</div>
      <div class="text-sm text-gray-500">Loading…</div>
    </div>
  </div>
{:else if $currentRoute === '/setup'}
  <Setup />
{:else}
  <div class="flex h-screen overflow-hidden bg-gray-950">
    <Sidebar />
    <main class="flex-1 overflow-y-auto">
      {#if routes[$currentRoute]}
        <svelte:component this={routes[$currentRoute]} />
      {:else}
        <Dashboard />
      {/if}
    </main>
  </div>
{/if}
