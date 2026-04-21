use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

use crate::store::AppStore;
use crate::tauri::{self, CreateSourceInput};
use crate::types::EmissionSource;

struct Category {
    num: i32,
    name: &'static str,
    dir: &'static str,
    hint: &'static str,
}

const CATEGORIES: &[Category] = &[
    Category { num: 1,  name: "Purchased goods and services",              dir: "upstream",   hint: "Cradle-to-gate emissions from goods/services purchased in the reporting year" },
    Category { num: 2,  name: "Capital goods",                             dir: "upstream",   hint: "Extraction, production, and transport of capital goods purchased/acquired" },
    Category { num: 3,  name: "Fuel- and energy-related activities",       dir: "upstream",   hint: "Upstream emissions from fuel/energy not in Scope 1 or 2 (extraction, refining, T&D losses)" },
    Category { num: 4,  name: "Upstream transportation and distribution",  dir: "upstream",   hint: "Transport of purchased products between suppliers and company" },
    Category { num: 5,  name: "Waste generated in operations",             dir: "upstream",   hint: "Disposal and treatment of waste generated in the reporting year" },
    Category { num: 6,  name: "Business travel",                           dir: "upstream",   hint: "Transportation by employees for business in vehicles not owned/controlled by company" },
    Category { num: 7,  name: "Employee commuting",                        dir: "upstream",   hint: "Transportation of employees between home and work in non-company vehicles" },
    Category { num: 8,  name: "Upstream leased assets",                    dir: "upstream",   hint: "Operation of assets leased by the reporting company (not in Scope 1/2)" },
    Category { num: 9,  name: "Downstream transportation and distribution",dir: "downstream", hint: "Transport of sold products after point of sale" },
    Category { num: 10, name: "Processing of sold products",               dir: "downstream", hint: "Processing of intermediate products sold by company" },
    Category { num: 11, name: "Use of sold products",                      dir: "downstream", hint: "End-use of goods and services sold in the reporting year" },
    Category { num: 12, name: "End-of-life treatment of sold products",    dir: "downstream", hint: "Waste disposal and treatment of sold products at end of life" },
    Category { num: 13, name: "Downstream leased assets",                  dir: "downstream", hint: "Operation of assets owned by the company and leased to others" },
    Category { num: 14, name: "Franchises",                                dir: "downstream", hint: "Operation of franchises not included in Scope 1/2" },
    Category { num: 15, name: "Investments",                               dir: "downstream", hint: "Investment activities (equity, debt, project finance) for finance/insurance sectors" },
];

const GHG_TYPES: &[&str] = &[
    "CO2", "CH4_non_fossil", "CH4_fossil", "N2O", "HFC", "PFC", "SF6", "NF3", "other",
];

#[component]
pub fn Scope3() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let by_category = RwSignal::new(HashMap::<i32, Vec<EmissionSource>>::new());
    let entities = RwSignal::new(Vec::<(i64, String)>::new());
    let open_form = RwSignal::new(None::<i32>);
    let open_exclude = RwSignal::new(None::<i32>);
    let exclude_reasons = RwSignal::new(HashMap::<i32, String>::new());
    let error = RwSignal::new(String::new());

    // Shared form state
    let f_entity_id = RwSignal::new(0i64);
    let f_activity_value = RwSignal::new(0.0f64);
    let f_activity_unit = RwSignal::new("t".to_string());
    let f_activity_source = RwSignal::new("Supplier Report".to_string());
    let f_ghg_type = RwSignal::new("CO2".to_string());
    let f_ef_value = RwSignal::new(0.0f64);
    let f_ef_unit = RwSignal::new("kgCO2e/t".to_string());
    let f_ef_source = RwSignal::new("GHG Protocol".to_string());
    let f_gwp = RwSignal::new(1.0f64);
    let f_uncertainty = RwSignal::new(15.0f64);
    let f_notes = RwSignal::new(String::new());

    let _reload = move || {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                if let Ok(all) = tauri::list_sources(period.id, Some(3)).await {
                    let mut grouped: HashMap<i32, Vec<EmissionSource>> = (1..=15).map(|i| (i, vec![])).collect();
                    for s in all {
                        if let Some(cat) = s.scope3_category {
                            grouped.entry(cat).or_default().push(s);
                        }
                    }
                    by_category.set(grouped);
                }
            });
        }
    };

    Effect::new(move |_| {
        let period = store.active_period.get();
        let org = store.active_org.get();
        if let (Some(period), Some(org)) = (period, org) {
            spawn_local(async move {
                if let Ok(all) = tauri::list_sources(period.id, Some(3)).await {
                    let mut grouped: HashMap<i32, Vec<EmissionSource>> = (1..=15).map(|i| (i, vec![])).collect();
                    for s in all {
                        if let Some(cat) = s.scope3_category {
                            grouped.entry(cat).or_default().push(s);
                        }
                    }
                    by_category.set(grouped);
                }
                if let Ok(ents) = tauri::list_entities(org.id).await {
                    let mapped: Vec<_> = ents.into_iter().map(|e| (e.id, e.name)).collect();
                    if let Some(first) = mapped.first() {
                        f_entity_id.set(first.0);
                    }
                    entities.set(mapped);
                }
            });
        }
    });

    let add_source = move |cat_num: i32, cat_name: &'static str| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        error.set(String::new());
        let input = CreateSourceInput {
            entity_id: f_entity_id.get(),
            period_id: period.id,
            scope: 3,
            scope2_method: None,
            scope3_category: Some(cat_num),
            category_name: cat_name.into(),
            ghg_type: f_ghg_type.get(),
            activity_value: f_activity_value.get(),
            activity_unit: f_activity_unit.get(),
            activity_source: Some(f_activity_source.get()),
            emission_factor_value: f_ef_value.get(),
            emission_factor_unit: f_ef_unit.get(),
            emission_factor_source: f_ef_source.get(),
            emission_factor_citation: None,
            gwp_value: f_gwp.get(),
            biogenic_co2_tco2e: None,
            uncertainty_pct: Some(f_uncertainty.get()),
            notes: { let n = f_notes.get(); if n.is_empty() { None } else { Some(n) } },
        };
        spawn_local(async move {
            match tauri::create_source(input).await {
                Ok(_) => {
                    if let Ok(all) = tauri::list_sources(period.id, Some(3)).await {
                        let mut grouped: HashMap<i32, Vec<EmissionSource>> = (1..=15).map(|i| (i, vec![])).collect();
                        for s in all {
                            if let Some(cat) = s.scope3_category {
                                grouped.entry(cat).or_default().push(s);
                            }
                        }
                        by_category.set(grouped);
                    }
                    open_form.set(None);
                }
                Err(e) => error.set(e),
            }
        });
    };

    let mark_excluded = move |cat_num: i32, cat_name: &'static str| {
        let period = store.active_period.get();
        let reason = exclude_reasons.with(|r| r.get(&cat_num).cloned().unwrap_or_default());
        if period.is_none() || reason.trim().is_empty() {
            error.set("Exclusion reason is required (ISO 14064-1)".into());
            return;
        }
        let period = period.unwrap();
        let entity_id = entities.with(|e| e.first().map(|(id, _)| *id).unwrap_or(0));
        error.set(String::new());
        let input = CreateSourceInput {
            entity_id,
            period_id: period.id,
            scope: 3,
            scope2_method: None,
            scope3_category: Some(cat_num),
            category_name: cat_name.into(),
            ghg_type: "CO2".into(),
            activity_value: 0.0,
            activity_unit: "n/a".into(),
            activity_source: Some("Excluded".into()),
            emission_factor_value: 0.0,
            emission_factor_unit: "n/a".into(),
            emission_factor_source: "n/a".into(),
            emission_factor_citation: None,
            gwp_value: 1.0,
            biogenic_co2_tco2e: None,
            uncertainty_pct: None,
            notes: Some(format!("EXCLUDED: {reason}")),
        };
        spawn_local(async move {
            match tauri::create_source(input).await {
                Ok(_) => {
                    if let Ok(all) = tauri::list_sources(period.id, Some(3)).await {
                        let mut grouped: HashMap<i32, Vec<EmissionSource>> = (1..=15).map(|i| (i, vec![])).collect();
                        for s in all {
                            if let Some(cat) = s.scope3_category {
                                grouped.entry(cat).or_default().push(s);
                            }
                        }
                        by_category.set(grouped);
                    }
                    open_exclude.set(None);
                }
                Err(e) => error.set(e),
            }
        });
    };

    let remove = move |id: i64| {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                let _ = tauri::delete_source(id, "User deleted").await;
                if let Ok(all) = tauri::list_sources(period.id, Some(3)).await {
                    let mut grouped: HashMap<i32, Vec<EmissionSource>> = (1..=15).map(|i| (i, vec![])).collect();
                    for s in all {
                        if let Some(cat) = s.scope3_category {
                            grouped.entry(cat).or_default().push(s);
                        }
                    }
                    by_category.set(grouped);
                }
            });
        }
    };

    let input_cls = "w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none";
    let label_cls = "mb-1 block text-xs font-medium text-gray-400";

    view! {
        <div class="p-8">
            <div class="mb-6">
                <h1 class="text-xl font-bold text-gray-100">"Scope 3 — Other Indirect Emissions"</h1>
                <p class="text-xs text-gray-500">"GRI 305-3 · ISO 14064-1 §5.3.3 · GHG Protocol Corporate Value Chain — 15 categories"</p>
            </div>

            <div class="mb-4 rounded-xl border border-yellow-800/40 bg-yellow-950/15 p-3">
                <p class="text-xs text-yellow-400">
                    <span class="font-semibold">"ISO 14064-1 requirement: "</span>
                    "All excluded categories must have a documented reason. Exclusion without a reason is not compliant."
                </p>
            </div>

            {move || (!error.get().is_empty()).then(|| view! {
                <p class="mb-4 text-xs text-red-400">{error.get()}</p>
            })}

            <div class="space-y-2">
                {CATEGORIES.iter().map(|cat| {
                    let cat_num = cat.num;
                    let cat_name = cat.name;
                    let cat_dir = cat.dir;
                    let cat_hint = cat.hint;

                    view! {
                        <div class=move || {
                            let bc = by_category.get();
                            let sources = bc.get(&cat_num).cloned().unwrap_or_default();
                            let excluded = sources.iter().any(|s| s.notes.as_deref().map(|n| n.starts_with("EXCLUDED:")).unwrap_or(false));
                            if excluded {
                                "overflow-hidden rounded-xl border border-gray-800/50 opacity-60 bg-gray-900"
                            } else {
                                "overflow-hidden rounded-xl border border-gray-800 bg-gray-900"
                            }
                        }>
                            // Category header
                            <div class="flex items-center justify-between px-4 py-3">
                                <div class="flex items-center gap-3">
                                    <span class="flex h-6 w-6 items-center justify-center rounded-full bg-gray-800 text-xs font-bold text-gray-400">
                                        {cat_num}
                                    </span>
                                    <div>
                                        <p class="text-sm font-medium text-gray-200">{cat_name}</p>
                                        <p class="text-xs text-gray-600">{cat_dir}" · "{cat_hint}</p>
                                    </div>
                                </div>
                                <div class="flex items-center gap-3">
                                    {move || {
                                        let bc = by_category.get();
                                        let sources = bc.get(&cat_num).cloned().unwrap_or_default();
                                        let total: f64 = sources.iter().map(|s| s.emissions_tco2e.unwrap_or(0.0)).sum();
                                        let excluded = sources.iter().any(|s| s.notes.as_deref().map(|n| n.starts_with("EXCLUDED:")).unwrap_or(false));
                                        view! {
                                            {if total > 0.0 {
                                                view! { <span class="text-sm font-semibold text-gray-200">{format!("{total:.3} tCO₂e")}</span> }.into_any()
                                            } else if excluded {
                                                view! { <span class="text-xs text-gray-600 italic">"excluded"</span> }.into_any()
                                            } else {
                                                view! { <span class="text-xs text-gray-600">"not entered"</span> }.into_any()
                                            }}
                                            {(!excluded).then(|| view! {
                                                <button
                                                    on:click=move |_| open_form.update(|f| *f = if *f == Some(cat_num) { None } else { Some(cat_num) })
                                                    class="rounded-lg bg-green-700/60 px-3 py-1 text-xs font-medium text-green-300 hover:bg-green-700"
                                                >"+ Add"</button>
                                                <button
                                                    on:click=move |_| open_exclude.update(|f| *f = if *f == Some(cat_num) { None } else { Some(cat_num) })
                                                    class="rounded-lg border border-gray-700 px-3 py-1 text-xs text-gray-500 hover:border-gray-600 hover:text-gray-400"
                                                >"Exclude"</button>
                                            })}
                                        }
                                    }}
                                </div>
                            </div>

                            // Existing sources mini-table
                            {move || {
                                let bc = by_category.get();
                                let sources = bc.get(&cat_num).cloned().unwrap_or_default();
                                let excluded = sources.iter().any(|s| s.notes.as_deref().map(|n| n.starts_with("EXCLUDED:")).unwrap_or(false));
                                (!sources.is_empty() && !excluded).then(|| view! {
                                    <div class="border-t border-gray-800/50 px-4 pb-2">
                                        <table class="w-full text-xs">
                                            <thead>
                                                <tr class="text-gray-600">
                                                    <th class="py-1 text-left font-medium">"Activity"</th>
                                                    <th class="py-1 text-left font-medium">"EF"</th>
                                                    <th class="py-1 text-left font-medium">"GHG"</th>
                                                    <th class="py-1 text-left font-medium">"tCO₂e"</th>
                                                    <th class="py-1"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {sources.into_iter().map(|s| {
                                                    let id = s.id;
                                                    view! {
                                                        <tr class="text-gray-400">
                                                            <td class="py-0.5">{s.activity_value}" "{s.activity_unit}</td>
                                                            <td class="py-0.5">{s.emission_factor_value}" "{s.emission_factor_unit}</td>
                                                            <td class="py-0.5 font-mono">{s.ghg_type}</td>
                                                            <td class="py-0.5 font-semibold text-gray-300">
                                                                {s.emissions_tco2e.map(|v| format!("{v:.3}")).unwrap_or_else(|| "—".into())}
                                                            </td>
                                                            <td class="py-0.5 text-right">
                                                                <button on:click=move |_| remove(id) class="text-red-600 hover:text-red-400">"✕"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                })
                            }}

                            // Add form
                            {move || (open_form.get() == Some(cat_num)).then(|| view! {
                                <div class="border-t border-gray-700 bg-gray-900/60 p-4">
                                    <div class="grid gap-3 sm:grid-cols-3">
                                        <div>
                                            <label class=label_cls>"Activity value"</label>
                                            <input type="number" step="0.001"
                                                prop:value=move || f_activity_value.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_activity_value.set(v); } }
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"Activity unit"</label>
                                            <input type="text"
                                                prop:value=move || f_activity_unit.get()
                                                on:input=move |ev| f_activity_unit.set(event_target_value(&ev))
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"Data source"</label>
                                            <select prop:value=move || f_activity_source.get() on:change=move |ev| f_activity_source.set(event_target_value(&ev)) class=input_cls>
                                                <option>"Supplier Report"</option>
                                                <option>"Invoice"</option>
                                                <option>"Spend-based"</option>
                                                <option>"Estimate"</option>
                                                <option>"Average data"</option>
                                            </select>
                                        </div>
                                        <div>
                                            <label class=label_cls>"GHG type"</label>
                                            <select prop:value=move || f_ghg_type.get() on:change=move |ev| f_ghg_type.set(event_target_value(&ev)) class=input_cls>
                                                {GHG_TYPES.iter().map(|g| view! { <option>{*g}</option> }).collect_view()}
                                            </select>
                                        </div>
                                        <div>
                                            <label class=label_cls>"Emission factor"</label>
                                            <input type="number" step="0.0001"
                                                prop:value=move || f_ef_value.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_ef_value.set(v); } }
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"EF unit"</label>
                                            <input type="text"
                                                prop:value=move || f_ef_unit.get()
                                                on:input=move |ev| f_ef_unit.set(event_target_value(&ev))
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"EF source"</label>
                                            <input type="text"
                                                prop:value=move || f_ef_source.get()
                                                on:input=move |ev| f_ef_source.set(event_target_value(&ev))
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"GWP value"</label>
                                            <input type="number" step="0.1"
                                                prop:value=move || f_gwp.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_gwp.set(v); } }
                                                class=input_cls />
                                        </div>
                                        <div>
                                            <label class=label_cls>"Uncertainty (%)"</label>
                                            <input type="number" step="1" min="0" max="100"
                                                prop:value=move || f_uncertainty.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_uncertainty.set(v); } }
                                                class=input_cls />
                                        </div>
                                        <div class="sm:col-span-3">
                                            <label class=label_cls>"Notes"</label>
                                            <input type="text"
                                                prop:value=move || f_notes.get()
                                                on:input=move |ev| f_notes.set(event_target_value(&ev))
                                                class=input_cls />
                                        </div>
                                    </div>
                                    <div class="mt-3 flex gap-2">
                                        <button on:click=move |_| add_source(cat_num, cat_name)
                                            class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">
                                            "Save"
                                        </button>
                                        <button on:click=move |_| open_form.set(None)
                                            class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">
                                            "Cancel"
                                        </button>
                                    </div>
                                </div>
                            })}

                            // Exclude form
                            {move || (open_exclude.get() == Some(cat_num)).then(|| view! {
                                <div class="border-t border-gray-700 bg-gray-900/60 p-4">
                                    <p class="mb-2 text-xs text-yellow-500">
                                        "ISO 14064-1 requires a documented reason for excluding any Scope 3 category."
                                    </p>
                                    <div class="flex gap-2">
                                        <input
                                            type="text"
                                            placeholder="e.g. Not material — category represents <1% of total emissions"
                                            prop:value=move || exclude_reasons.with(|r| r.get(&cat_num).cloned().unwrap_or_default())
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                exclude_reasons.update(|r| { r.insert(cat_num, val); });
                                            }
                                            class="input flex-1 w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none"
                                        />
                                        <button on:click=move |_| mark_excluded(cat_num, cat_name)
                                            class="rounded-lg border border-yellow-700 px-4 py-2 text-sm text-yellow-500 hover:border-yellow-600">
                                            "Confirm"
                                        </button>
                                        <button on:click=move |_| open_exclude.set(None)
                                            class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-500 hover:border-gray-600">
                                            "Cancel"
                                        </button>
                                    </div>
                                </div>
                            })}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
