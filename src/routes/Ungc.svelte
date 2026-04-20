<script lang="ts">
  import { onMount } from 'svelte'
  import { activePeriod, activeOrg } from '$lib/stores/app'
  import { initCop, autoPopulateCop, getCopQuestions, saveCopResponse, signCeoStatement, computeComplianceLevel } from '$lib/tauri'

  type Question = {
    question_id: string
    question_text: string
    response: string | null
    auto_populated: boolean
    source_table: string | null
  }

  type Cop = {
    id: number
    org_id: number
    reporting_year: number
    status: string
    compliance_level: string | null
    ceo_statement_signed: boolean
    submitted_at: number | null
  }

  let cop = $state<Cop | null>(null)
  let questions = $state<Question[]>([])
  let complianceLevel = $state<string | null>(null)
  let ceoName = $state('')
  let loading = $state(false)
  let saving = $state<string | null>(null)  // question_id being saved
  let error = $state('')
  let autoPopulating = $state(false)

  // Group questions by section prefix (E=Environment, G=Governance, L=Labour, H=Human Rights)
  const SECTION_LABELS: Record<string, string> = {
    E: 'Environment (Principles 7–9)',
    G: 'Governance & Anti-Corruption (Principle 10)',
    L: 'Labour Standards (Principles 3–6)',
    H: 'Human Rights (Principles 1–2)',
  }

  type QuestionsBySection = Record<string, Question[]>

  let bySection = $derived<QuestionsBySection>(() => {
    const result: QuestionsBySection = {}
    for (const q of questions) {
      const section = q.question_id[0] ?? 'O'
      if (!result[section]) result[section] = []
      result[section].push(q)
    }
    return result
  })

  onMount(async () => {
    const period = $activePeriod
    const org = $activeOrg
    if (!period || !org) return
    loading = true
    error = ''
    try {
      const result = await initCop(org.id, period.year) as Cop
      cop = result
      questions = (await getCopQuestions(result.id)) as Question[]
      const level = await computeComplianceLevel(result.id)
      complianceLevel = level
    } catch (e) { error = String(e) }
    finally { loading = false }
  })

  async function populate() {
    const period = $activePeriod
    if (!cop || !period) return
    autoPopulating = true
    error = ''
    try {
      const n = await autoPopulateCop(cop.id, period.id)
      questions = (await getCopQuestions(cop.id)) as Question[]
      complianceLevel = await computeComplianceLevel(cop.id)
    } catch (e) { error = String(e) }
    finally { autoPopulating = false }
  }

  async function saveResponse(q: Question, value: string) {
    if (!cop) return
    saving = q.question_id
    try {
      await saveCopResponse(cop.id, q.question_id, value)
      // Update local state optimistically
      questions = questions.map(x => x.question_id === q.question_id ? { ...x, response: value } : x)
      complianceLevel = await computeComplianceLevel(cop.id)
    } catch (e) { error = String(e) }
    finally { saving = null }
  }

  async function signStatement() {
    if (!cop || !ceoName.trim()) { error = 'CEO name is required'; return }
    error = ''
    try {
      await signCeoStatement(cop.id, ceoName.trim())
      cop = { ...cop, ceo_statement_signed: true }
      complianceLevel = await computeComplianceLevel(cop.id)
    } catch (e) { error = String(e) }
  }

  function levelColor(level: string | null) {
    switch (level) {
      case 'lead':     return 'text-purple-400 border-purple-800 bg-purple-950/30'
      case 'advanced': return 'text-blue-400 border-blue-800 bg-blue-950/30'
      case 'active':   return 'text-green-400 border-green-800 bg-green-950/30'
      default:         return 'text-yellow-400 border-yellow-800 bg-yellow-950/30'
    }
  }

  function answeredCount() {
    return questions.filter(q => q.response && q.response.trim().length > 0).length
  }
</script>

<div class="p-8">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-100">UNGC Communication on Progress</h1>
    <p class="text-xs text-gray-500">
      Annual submission · Principles 7–9 (Environment) · 2025 questionnaire format
    </p>
  </div>

  {#if loading}
    <p class="text-sm text-gray-500">Loading COP…</p>
  {:else if error}
    <div class="mb-4 rounded-xl border border-red-800 bg-red-950/20 p-4 text-sm text-red-400">{error}</div>
  {/if}

  {#if cop}
    <!-- Status bar -->
    <div class="mb-6 flex items-center gap-4">
      {#if complianceLevel}
        <span class="rounded-full border px-3 py-1 text-xs font-semibold uppercase {levelColor(complianceLevel)}">
          {complianceLevel}
        </span>
      {/if}
      <span class="text-xs text-gray-500">
        {answeredCount()} / {questions.length} questions answered
      </span>
      {#if cop.ceo_statement_signed}
        <span class="text-xs text-green-500">✓ CEO Statement signed</span>
      {/if}
      <div class="ml-auto flex gap-2">
        <button onclick={populate} disabled={autoPopulating}
          class="rounded-lg border border-gray-700 px-4 py-2 text-xs text-gray-400 hover:border-gray-600 disabled:opacity-50">
          {autoPopulating ? 'Populating…' : 'Auto-populate from GRI 305'}
        </button>
      </div>
    </div>

    <!-- CEO Statement (required before submission) -->
    {#if !cop.ceo_statement_signed}
      <div class="mb-6 rounded-xl border border-yellow-700/50 bg-yellow-950/20 p-5">
        <h2 class="mb-3 text-sm font-semibold text-yellow-400">CEO Statement of Continued Support</h2>
        <p class="mb-4 text-xs text-yellow-300/70">
          Required for all UNGC COP submissions. The CEO (or equivalent) must affirm continued commitment to
          the Ten Principles. This cannot be submitted without a signed statement.
        </p>
        <div class="flex gap-3">
          <input bind:value={ceoName} type="text" placeholder="CEO full name"
            class="flex-1 rounded-lg border border-yellow-700/50 bg-gray-800 px-3 py-2 text-sm text-gray-100
                   placeholder:text-gray-600 focus:border-yellow-600 focus:outline-none" />
          <button onclick={signStatement}
            class="rounded-lg bg-yellow-600 px-4 py-2 text-sm font-semibold text-white hover:bg-yellow-700">
            Sign Statement
          </button>
        </div>
        <p class="mt-3 text-[10px] text-gray-600">
          By signing, {ceoName || '[CEO name]'} confirms continued commitment to the UNGC Ten Principles as of {new Date().getFullYear()}.
        </p>
      </div>
    {:else}
      <div class="mb-6 rounded-xl border border-green-800/40 bg-green-950/15 p-4">
        <p class="text-sm text-green-400">✓ CEO Statement signed — ready for submission</p>
      </div>
    {/if}

    <!-- Question sections -->
    {#each Object.entries(bySection()) as [section, qs]}
      <div class="mb-6">
        <h2 class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">
          {SECTION_LABELS[section] ?? section}
        </h2>
        <div class="space-y-2">
          {#each qs as q}
            <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
              <div class="mb-2 flex items-start justify-between gap-4">
                <div class="flex items-start gap-2">
                  <span class="mt-0.5 font-mono text-xs text-gray-600">{q.question_id}</span>
                  <p class="text-sm text-gray-300">{q.question_text}</p>
                </div>
                {#if q.auto_populated}
                  <span class="shrink-0 rounded-full border border-green-800/50 bg-green-950/30 px-2 py-0.5 text-[10px] text-green-500">
                    auto
                  </span>
                {/if}
              </div>
              <textarea
                value={q.response ?? ''}
                onblur={(e) => saveResponse(q, (e.target as HTMLTextAreaElement).value)}
                rows="3"
                placeholder="Enter your response…"
                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100
                       placeholder:text-gray-600 focus:border-green-600 focus:outline-none resize-none"
              ></textarea>
              {#if saving === q.question_id}
                <p class="mt-1 text-[10px] text-gray-600">Saving…</p>
              {:else if q.response}
                <p class="mt-1 text-[10px] text-green-700">✓ Answered</p>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/each}

    <!-- Compliance level explainer -->
    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
      <h2 class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">Compliance Levels</h2>
      <div class="space-y-2">
        {#each [
          { level: 'beginner', desc: 'CEO Statement + any responses submitted', color: 'text-yellow-400' },
          { level: 'active',   desc: 'CEO Statement + ≥50% of questions answered', color: 'text-green-400' },
          { level: 'advanced', desc: 'CEO Statement + ≥75% answered + environmental & social coverage', color: 'text-blue-400' },
          { level: 'lead',     desc: 'CEO Statement + ≥90% answered + all principle areas covered', color: 'text-purple-400' },
        ] as l}
          <div class="flex items-start gap-3">
            <span class="w-16 shrink-0 text-xs font-semibold uppercase {l.color}">{l.level}</span>
            <span class="text-xs text-gray-500">{l.desc}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
