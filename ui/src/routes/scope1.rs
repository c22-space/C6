use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::store::AppStore;
use crate::tauri::{self, CreateSourceInput};
use crate::types::EmissionSource;

const GHG_TYPES: &[&str] = &[
    "CO2", "CH4_non_fossil", "CH4_fossil", "N2O", "HFC", "PFC", "SF6", "NF3", "other",
];

const SCOPE1_CATEGORIES: &[&str] = &[
    "Stationary combustion — natural gas",
    "Stationary combustion — diesel",
    "Stationary combustion — LPG",
    "Stationary combustion — coal",
    "Mobile combustion — company vehicles (petrol)",
    "Mobile combustion — company vehicles (diesel)",
    "Fugitive emissions — refrigerants",
    "Fugitive emissions — methane (natural gas)",
    "Process emissions",
    "Other direct emissions",
];

fn fmt(n: Option<f64>) -> String {
    n.map(|v| format!("{v:.3}")).unwrap_or_else(|| "—".into())
}

#[component]
pub fn Scope1() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let sources = RwSignal::new(Vec::<EmissionSource>::new());
    let entities = RwSignal::new(Vec::<(i64, String)>::new());
    let show_form = RwSignal::new(false);
    let error = RwSignal::new(String::new());

    // Form state
    let f_entity_id = RwSignal::new(0i64);
    let f_category = RwSignal::new(String::new());
    let f_ghg_type = RwSignal::new("CO2".to_string());
    let f_activity_value = RwSignal::new(0.0f64);
    let f_activity_unit = RwSignal::new("kWh".to_string());
    let f_activity_source = RwSignal::new("Invoice".to_string());
    let f_ef_value = RwSignal::new(0.0f64);
    let f_ef_unit = RwSignal::new("kgCO2e/kWh".to_string());
    let f_ef_source = RwSignal::new("DEFRA 2024".to_string());
    let f_gwp = RwSignal::new(1.0f64);
    let f_uncertainty = RwSignal::new(5.0f64);
    let f_notes = RwSignal::new(String::new());

    let _reload = move || {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                if let Ok(s) = tauri::list_sources(period.id, Some(1)).await {
                    sources.set(s);
                }
            });
        }
    };

    Effect::new(move |_| {
        let period = store.active_period.get();
        let org = store.active_org.get();
        if let (Some(period), Some(org)) = (period, org) {
            spawn_local(async move {
                if let Ok(s) = tauri::list_sources(period.id, Some(1)).await {
                    sources.set(s);
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

    let add_source = move |_| {
        let period = store.active_period.get();
        if period.is_none() {
            return;
        }
        let period = period.unwrap();
        error.set(String::new());
        let input = CreateSourceInput {
            entity_id: f_entity_id.get(),
            period_id: period.id,
            scope: 1,
            scope2_method: None,
            scope3_category: None,
            category_name: f_category.get(),
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
            notes: {
                let n = f_notes.get();
                if n.is_empty() { None } else { Some(n) }
            },
        };
        spawn_local(async move {
            match tauri::create_source(input).await {
                Ok(_) => {
                    if let Ok(s) = tauri::list_sources(period.id, Some(1)).await {
                        sources.set(s);
                    }
                    show_form.set(false);
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
                if let Ok(s) = tauri::list_sources(period.id, Some(1)).await {
                    sources.set(s);
                }
            });
        }
    };

    let input_cls = "w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none";
    let label_cls = "mb-1 block text-xs font-medium text-gray-400";

    view! {
        <div class="p-8">
            <div class="mb-6 flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-bold text-gray-100">"Scope 1 — Direct Emissions"</h1>
                    <p class="text-xs text-gray-500">"GRI 305-1 · ISO 14064-1 §5.3.1 · Owned/controlled sources"</p>
                </div>
                <button
                    on:click=move |_| show_form.update(|v| *v = !*v)
                    class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700"
                >
                    "+ Add source"
                </button>
            </div>

            {move || show_form.get().then(|| view! {
                <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
                    <h3 class="mb-4 text-sm font-semibold text-gray-200">"Add Scope 1 Source"</h3>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <div>
                            <label class=label_cls>"Category"</label>
                            <select
                                prop:value=move || f_category.get()
                                on:change=move |ev| f_category.set(event_target_value(&ev))
                                class=input_cls
                            >
                                <option value="">"Select category…"</option>
                                {SCOPE1_CATEGORIES.iter().map(|c| view! {
                                    <option>{*c}</option>
                                }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class=label_cls>"GHG Type"</label>
                            <select
                                prop:value=move || f_ghg_type.get()
                                on:change=move |ev| f_ghg_type.set(event_target_value(&ev))
                                class=input_cls
                            >
                                {GHG_TYPES.iter().map(|g| view! { <option>{*g}</option> }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class=label_cls>"Activity value"</label>
                            <input type="number" step="0.001"
                                prop:value=move || f_activity_value.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_activity_value.set(v); }
                                }
                                class=input_cls
                            />
                        </div>
                        <div>
                            <label class=label_cls>"Activity unit"</label>
                            <input type="text" placeholder="kWh, L, m3, kg…"
                                prop:value=move || f_activity_unit.get()
                                on:input=move |ev| f_activity_unit.set(event_target_value(&ev))
                                class=input_cls
                            />
                        </div>
                        <div>
                            <label class=label_cls>"Emission factor (kgCO₂e/unit)"</label>
                            <input type="number" step="0.0001"
                                prop:value=move || f_ef_value.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_ef_value.set(v); }
                                }
                                class=input_cls
                            />
                        </div>
                        <div>
                            <label class=label_cls>"EF source"</label>
                            <input type="text"
                                prop:value=move || f_ef_source.get()
                                on:input=move |ev| f_ef_source.set(event_target_value(&ev))
                                class=input_cls
                            />
                        </div>
                        <div>
                            <label class=label_cls>"GWP value"</label>
                            <input type="number" step="0.1"
                                prop:value=move || f_gwp.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_gwp.set(v); }
                                }
                                class=input_cls
                            />
                        </div>
                        <div>
                            <label class=label_cls>"Uncertainty (%)"</label>
                            <input type="number" step="1" min="0" max="100"
                                prop:value=move || f_uncertainty.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() { f_uncertainty.set(v); }
                                }
                                class=input_cls
                            />
                        </div>
                        <div class="sm:col-span-2">
                            <label class=label_cls>"Notes"</label>
                            <input type="text"
                                prop:value=move || f_notes.get()
                                on:input=move |ev| f_notes.set(event_target_value(&ev))
                                class=input_cls
                            />
                        </div>
                    </div>
                    {move || (!error.get().is_empty()).then(|| view! {
                        <p class="mt-2 text-xs text-red-400">{error.get()}</p>
                    })}
                    <div class="mt-4 flex gap-2">
                        <button on:click=add_source
                            class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">
                            "Save"
                        </button>
                        <button on:click=move |_| show_form.set(false)
                            class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">
                            "Cancel"
                        </button>
                    </div>
                </div>
            })}

            // Sources table
            <div class="overflow-hidden rounded-xl border border-gray-800">
                <table class="w-full text-sm">
                    <thead class="border-b border-gray-800 bg-gray-900/60">
                        <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
                            <th class="px-4 py-3">"Category"</th>
                            <th class="px-4 py-3">"GHG"</th>
                            <th class="px-4 py-3">"Activity"</th>
                            <th class="px-4 py-3">"EF"</th>
                            <th class="px-4 py-3">"GWP"</th>
                            <th class="px-4 py-3">"tCO₂e"</th>
                            <th class="px-4 py-3">"±%"</th>
                            <th class="px-4 py-3"></th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800 bg-gray-900">
                        {move || {
                            let src = sources.get();
                            if src.is_empty() {
                                view! {
                                    <tr>
                                        <td colspan="8" class="px-4 py-8 text-center text-sm text-gray-500">
                                            "No Scope 1 sources yet."
                                        </td>
                                    </tr>
                                }.into_any()
                            } else {
                                src.into_iter().map(|s| {
                                    let id = s.id;
                                    view! {
                                        <tr class="hover:bg-gray-800/40">
                                            <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
                                            <td class="px-4 py-3 font-mono text-xs text-gray-400">{s.ghg_type}</td>
                                            <td class="px-4 py-3 text-gray-300">
                                                {s.activity_value}" "{s.activity_unit}
                                            </td>
                                            <td class="px-4 py-3 text-xs text-gray-500">
                                                {s.emission_factor_value}" "{s.emission_factor_unit}
                                            </td>
                                            <td class="px-4 py-3 text-xs text-gray-500">{s.gwp_value}</td>
                                            <td class="px-4 py-3 font-semibold text-gray-200">
                                                {fmt(s.emissions_tco2e)}
                                            </td>
                                            <td class="px-4 py-3 text-xs text-gray-500">
                                                {s.uncertainty_pct.map(|u| format!("±{u}%")).unwrap_or_else(|| "—".into())}
                                            </td>
                                            <td class="px-4 py-3">
                                                <button
                                                    on:click=move |_| remove(id)
                                                    class="text-xs text-red-500 hover:text-red-400"
                                                >"✕"</button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
