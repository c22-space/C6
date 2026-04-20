import { writable, derived } from 'svelte/store'
import type { Organization, ReportingPeriod } from '$lib/tauri'

export const activeOrg = writable<Organization | null>(null)
export const activePeriod = writable<ReportingPeriod | null>(null)
export const currentRoute = writable<string>('/')

export const isConfigured = derived(activeOrg, ($org) => $org !== null)
