use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::store::AppStore;
use crate::tauri;

#[component]
pub fn Setup() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");

    let step = RwSignal::new(1u32);
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    // Step 1: Org
    let org_name = RwSignal::new(String::new());
    let boundary_method = RwSignal::new("operational_control".to_string());
    let base_year = RwSignal::new(2024i32);
    let currency = RwSignal::new("USD".to_string());

    // Step 2: Reporting period
    let period_year = RwSignal::new(2024i32);
    let gwp_ar = RwSignal::new("AR6".to_string());

    let create_org_step = move |_| {
        let name = org_name.get();
        if name.trim().is_empty() {
            error.set("Organisation name is required".into());
            return;
        }
        loading.set(true);
        error.set(String::new());
        let bm = boundary_method.get();
        let by = base_year.get();
        let cur = currency.get();
        spawn_local(async move {
            match tauri::create_org(name.trim(), &bm, Some(by), &cur).await {
                Ok(org) => {
                    let org_id = org.id;
                    let org_name_clone = org.name.clone();
                    store.active_org.set(Some(org));
                    let _ = tauri::create_entity(
                        org_id,
                        &org_name_clone,
                        "parent",
                        None,
                        true,
                        true,
                        None,
                    )
                    .await;
                    step.set(2);
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    };

    let create_period_step = move |_| {
        let org = store.active_org.get();
        if org.is_none() {
            return;
        }
        let org = org.unwrap();
        loading.set(true);
        error.set(String::new());
        let py = period_year.get();
        let ar = gwp_ar.get();
        spawn_local(async move {
            match tauri::create_period(org.id, py, &ar).await {
                Ok(period) => {
                    store.active_period.set(Some(period));
                    store.navigate("/dashboard");
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    };

    let boundary_options = [
        ("operational_control", "Operational Control", "100% of entities where you control day-to-day operations (most common)"),
        ("financial_control",   "Financial Control",   "100% of entities where you control financial policies"),
        ("equity_share",        "Equity Share",        "Pro-rata share based on ownership percentage"),
    ];

    view! {
        <div class="flex min-h-screen items-center justify-center bg-gray-950 p-6">
            <div class="w-full max-w-lg">
                <div class="mb-8 text-center">
                    <div class="mb-2 text-3xl font-bold text-green-500">"C6"</div>
                    <p class="text-sm text-gray-500">"Carbon accounting for GRI 305, ISO 14064 & UNGC"</p>
                </div>

                // Step indicator
                <div class="mb-8 flex items-center gap-2">
                    {[1u32, 2u32].iter().map(|&s| view! {
                        <div class="flex items-center gap-2">
                            <div class=move || {
                                let cur = step.get();
                                if cur == s { "flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold bg-green-600 text-white" }
                                else if cur > s { "flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold bg-green-900 text-green-400" }
                                else { "flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold bg-gray-800 text-gray-500" }
                            }>
                                {s}
                            </div>
                            <span class=move || if step.get() == s { "text-xs text-gray-200" } else { "text-xs text-gray-600" }>
                                {if s == 1 { "Organisation" } else { "Reporting period" }}
                            </span>
                        </div>
                        {(s < 2).then(|| view! { <div class="flex-1 h-px bg-gray-800"></div> })}
                    }).collect_view()}
                </div>

                <div class="rounded-2xl border border-gray-800 bg-gray-900 p-6">
                    {move || if step.get() == 1 {
                        view! {
                            <div>
                                <h2 class="mb-5 text-base font-semibold text-gray-100">"Set up your organisation"</h2>
                                <div class="space-y-4">
                                    <div>
                                        <label class="mb-1.5 block text-xs font-medium text-gray-400">"Organisation name"</label>
                                        <input
                                            type="text"
                                            placeholder="Acme Corp"
                                            prop:value=move || org_name.get()
                                            on:input=move |ev| org_name.set(event_target_value(&ev))
                                            class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 placeholder:text-gray-600 focus:border-green-600 focus:outline-none focus:ring-1 focus:ring-green-600"
                                        />
                                    </div>

                                    <div>
                                        <label class="mb-1.5 block text-xs font-medium text-gray-400">
                                            "Organisational boundary method"
                                            <span class="ml-1 text-gray-600">"(ISO 14064-1 §5.2 — select one)"</span>
                                        </label>
                                        <div class="space-y-2">
                                            {boundary_options.iter().map(|(val, label, desc)| {
                                                let val = *val;
                                                view! {
                                                    <label class=move || {
                                                        if boundary_method.get() == val {
                                                            "flex cursor-pointer items-start gap-3 rounded-lg border p-3 transition-colors border-green-700 bg-green-950/30"
                                                        } else {
                                                            "flex cursor-pointer items-start gap-3 rounded-lg border p-3 transition-colors border-gray-800 hover:border-gray-700"
                                                        }
                                                    }>
                                                        <input
                                                            type="radio"
                                                            class="mt-0.5"
                                                            prop:checked=move || boundary_method.get() == val
                                                            on:change=move |_| boundary_method.set(val.to_string())
                                                        />
                                                        <div>
                                                            <div class="text-sm font-medium text-gray-200">{*label}</div>
                                                            <div class="text-xs text-gray-500">{*desc}</div>
                                                        </div>
                                                    </label>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="mb-1.5 block text-xs font-medium text-gray-400">"Base year"</label>
                                            <input
                                                type="number"
                                                min="2000"
                                                max="2030"
                                                prop:value=move || base_year.get()
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        base_year.set(v);
                                                    }
                                                }
                                                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none"
                                            />
                                        </div>
                                        <div>
                                            <label class="mb-1.5 block text-xs font-medium text-gray-400">"Currency"</label>
                                            <select
                                                prop:value=move || currency.get()
                                                on:change=move |ev| currency.set(event_target_value(&ev))
                                                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none"
                                            >
                                                <option>"USD"</option>
                                                <option>"EUR"</option>
                                                <option>"GBP"</option>
                                                <option>"AUD"</option>
                                                <option>"CAD"</option>
                                                <option>"JPY"</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>

                                {move || (!error.get().is_empty()).then(|| view! {
                                    <p class="mt-3 text-xs text-red-400">{error.get()}</p>
                                })}

                                <button
                                    on:click=create_org_step
                                    disabled=move || loading.get()
                                    class="mt-5 w-full rounded-lg bg-green-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-green-700 disabled:opacity-50 transition-colors"
                                >
                                    {move || if loading.get() { "Creating…" } else { "Continue →" }}
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <h2 class="mb-5 text-base font-semibold text-gray-100">"Create your first reporting period"</h2>
                                <div class="space-y-4">
                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="mb-1.5 block text-xs font-medium text-gray-400">"Reporting year"</label>
                                            <input
                                                type="number"
                                                min="2000"
                                                max="2030"
                                                prop:value=move || period_year.get()
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        period_year.set(v);
                                                    }
                                                }
                                                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none"
                                            />
                                        </div>
                                        <div>
                                            <label class="mb-1.5 block text-xs font-medium text-gray-400">
                                                "GWP values"
                                                <span class="ml-1 text-gray-600">"(IPCC AR)"</span>
                                            </label>
                                            <select
                                                prop:value=move || gwp_ar.get()
                                                on:change=move |ev| gwp_ar.set(event_target_value(&ev))
                                                class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none"
                                            >
                                                <option value="AR6">"AR6 (2021) — Recommended"</option>
                                                <option value="AR5">"AR5 (2013)"</option>
                                                <option value="AR4">"AR4 (2007)"</option>
                                            </select>
                                        </div>
                                    </div>

                                    <div class="rounded-lg border border-yellow-800/50 bg-yellow-950/20 p-3">
                                        <p class="text-xs text-yellow-500">
                                            <span class="font-semibold">"IPCC AR6 recommended. "</span>
                                            "AR6 values are current as of 2024 and required for new GRI 305 reports. Use AR4/AR5 only for historical comparisons."
                                        </p>
                                    </div>
                                </div>

                                {move || (!error.get().is_empty()).then(|| view! {
                                    <p class="mt-3 text-xs text-red-400">{error.get()}</p>
                                })}

                                <div class="mt-5 flex gap-3">
                                    <button
                                        on:click=move |_| step.set(1)
                                        class="rounded-lg border border-gray-700 px-4 py-2.5 text-sm font-medium text-gray-400 hover:border-gray-600 hover:text-gray-200 transition-colors"
                                    >
                                        "← Back"
                                    </button>
                                    <button
                                        on:click=create_period_step
                                        disabled=move || loading.get()
                                        class="flex-1 rounded-lg bg-green-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-green-700 disabled:opacity-50 transition-colors"
                                    >
                                        {move || if loading.get() { "Setting up…" } else { "Start accounting →" }}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>

                <p class="mt-6 text-center text-[10px] text-gray-700">
                    "Built by "
                    <a href="https://c22.space" class="hover:text-gray-500">"c22"</a>
                    " · "
                    <a href="https://c22.space/hire" class="hover:text-gray-500">"Hire us →"</a>
                </p>
            </div>
        </div>
    }
}
