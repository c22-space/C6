<script lang="ts">
  import { currentRoute, activeOrg, activePeriod } from '$lib/stores/app'

  const nav = [
    { path: '/dashboard',      label: 'Dashboard',      icon: '◈' },
    { path: '/sources/scope1', label: 'Scope 1',         icon: '①' },
    { path: '/sources/scope2', label: 'Scope 2',         icon: '②' },
    { path: '/sources/scope3', label: 'Scope 3',         icon: '③' },
    { path: '/reports',        label: 'Reports',         icon: '↗' },
    { path: '/ungc',           label: 'UNGC COP',        icon: '✦' },
    { path: '/settings',       label: 'Settings',        icon: '⚙' },
  ]

  function navigate(path: string) {
    currentRoute.set(path)
  }
</script>

<aside class="flex w-56 flex-col border-r border-gray-800 bg-gray-950 no-select">
  <!-- Logo -->
  <div class="flex h-14 items-center gap-2 border-b border-gray-800 px-4">
    <span class="text-lg font-bold text-green-500">c12</span>
    <span class="text-xs text-gray-600">Carbon Accounting</span>
  </div>

  <!-- Org / period selector -->
  <div class="border-b border-gray-800 px-3 py-3">
    {#if $activeOrg}
      <p class="truncate text-xs font-semibold text-gray-300">{$activeOrg.name}</p>
      {#if $activePeriod}
        <p class="text-xs text-gray-500">{$activePeriod.year} · {$activePeriod.gwp_ar_version}</p>
      {/if}
    {:else}
      <p class="text-xs text-gray-600">No organisation</p>
    {/if}
  </div>

  <!-- Navigation -->
  <nav class="flex-1 space-y-0.5 overflow-y-auto p-2">
    {#each nav as item}
      <button
        onclick={() => navigate(item.path)}
        class="sidebar-item w-full text-left {$currentRoute === item.path ? 'active' : ''}"
      >
        <span class="w-4 text-center font-mono text-xs">{item.icon}</span>
        {item.label}
      </button>
    {/each}
  </nav>

  <!-- Footer upsell — subtle c22 branding -->
  <div class="border-t border-gray-800 px-4 py-3">
    <p class="text-[10px] text-gray-700">
      Built by <a href="https://c22.space" target="_blank" rel="noopener"
        class="text-gray-600 hover:text-gray-400 transition-colors">c22</a>
      &nbsp;·&nbsp;
      <a href="https://c22.space/hire" target="_blank" rel="noopener"
        class="text-gray-600 hover:text-gray-400 transition-colors">Hire us →</a>
    </p>
  </div>
</aside>
