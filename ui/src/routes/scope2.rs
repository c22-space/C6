use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::store::AppStore;
use crate::tauri::{self, CreateSourceInput};
use crate::types::EmissionSource;

const SCOPE2_CATEGORIES: &[&str] = &[
    "Purchased electricity — grid",
    "Purchased electricity — renewable (REC/PPA)",
    "Purchased steam",
    "Purchased heat",
    "Purchased cooling",
    "District energy",
    "Other purchased energy",
];

const INSTRUMENT_TYPES: &[(&str, &str)] = &[
    ("none",  "No instrument (residual mix)"),
    ("REC",   "REC — Renewable Energy Certificate"),
    ("PPA",   "PPA — Power Purchase Agreement"),
    ("GG",    "GG — Green Gas/Guarantee of Origin"),
    ("other", "Other contractual instrument"),
];

fn fmt(n: f64) -> String {
    format!("{n:.3}")
}

fn total_tco2e(sources: &[EmissionSource]) -> f64 {
    sources.iter().map(|s| s.emissions_tco2e.unwrap_or(0.0)).sum()
}

#[derive(Clone)]
enum ShowForm {
    None,
    Location,
    Market,
}

#[component]
pub fn Scope2() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let location_sources = RwSignal::new(Vec::<EmissionSource>::new());
    let market_sources = RwSignal::new(Vec::<EmissionSource>::new());
    let entities = RwSignal::new(Vec::<(i64, String)>::new());
    let show_form = RwSignal::new(ShowForm::None);
    let error = RwSignal::new(String::new());

    // Location form state
    let loc_entity = RwSignal::new(0i64);
    let loc_category = RwSignal::new(String::new());
    let loc_activity_value = RwSignal::new(0.0f64);
    let loc_activity_unit = RwSignal::new("kWh".to_string());
    let loc_activity_source = RwSignal::new("Meter".to_string());
    let loc_ef_value = RwSignal::new(0.0f64);
    let loc_ef_source = RwSignal::new("National grid average".to_string());
    let loc_uncertainty = RwSignal::new(5.0f64);
    let loc_notes = RwSignal::new(String::new());

    // Market form state
    let mkt_entity = RwSignal::new(0i64);
    let mkt_category = RwSignal::new(String::new());
    let mkt_activity_value = RwSignal::new(0.0f64);
    let mkt_activity_unit = RwSignal::new("kWh".to_string());
    let mkt_activity_source = RwSignal::new("Invoice".to_string());
    let mkt_ef_value = RwSignal::new(0.0f64);
    let mkt_ef_unit = RwSignal::new("kgCO2e/kWh".to_string());
    let mkt_ef_source = RwSignal::new("Supplier-specific EF".to_string());
    let mkt_instrument = RwSignal::new("none".to_string());
    let mkt_uncertainty = RwSignal::new(5.0f64);
    let mkt_notes = RwSignal::new(String::new());

    let _reload = move || {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                if let Ok(all) = tauri::list_sources(period.id, Some(2)).await {
                    location_sources.set(all.iter().filter(|s| s.scope2_method.as_deref() == Some("location_based")).cloned().collect());
                    market_sources.set(all.into_iter().filter(|s| s.scope2_method.as_deref() == Some("market_based")).collect());
                }
            });
        }
    };

    Effect::new(move |_| {
        let period = store.active_period.get();
        let org = store.active_org.get();
        if let (Some(period), Some(org)) = (period, org) {
            spawn_local(async move {
                if let Ok(all) = tauri::list_sources(period.id, Some(2)).await {
                    location_sources.set(all.iter().filter(|s| s.scope2_method.as_deref() == Some("location_based")).cloned().collect());
                    market_sources.set(all.into_iter().filter(|s| s.scope2_method.as_deref() == Some("market_based")).collect());
                }
                if let Ok(ents) = tauri::list_entities(org.id).await {
                    let mapped: Vec<_> = ents.into_iter().map(|e| (e.id, e.name)).collect();
                    if let Some(first) = mapped.first() {
                        loc_entity.set(first.0);
                        mkt_entity.set(first.0);
                    }
                    entities.set(mapped);
                }
            });
        }
    });

    let add_location = move |_| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        error.set(String::new());
        let input = CreateSourceInput {
            entity_id: loc_entity.get(),
            period_id: period.id,
            scope: 2,
            scope2_method: Some("location_based".into()),
            scope3_category: None,
            category_name: loc_category.get(),
            ghg_type: "CO2".into(),
            activity_value: loc_activity_value.get(),
            activity_unit: loc_activity_unit.get(),
            activity_source: Some(loc_activity_source.get()),
            emission_factor_value: loc_ef_value.get(),
            emission_factor_unit: "kgCO2e/kWh".into(),
            emission_factor_source: loc_ef_source.get(),
            emission_factor_citation: None,
            gwp_value: 1.0,
            biogenic_co2_tco2e: None,
            uncertainty_pct: Some(loc_uncertainty.get()),
            notes: { let n = loc_notes.get(); if n.is_empty() { None } else { Some(n) } },
        };
        spawn_local(async move {
            match tauri::create_source(input).await {
                Ok(_) => {
                    if let Ok(all) = tauri::list_sources(period.id, Some(2)).await {
                        location_sources.set(all.iter().filter(|s| s.scope2_method.as_deref() == Some("location_based")).cloned().collect());
                        market_sources.set(all.into_iter().filter(|s| s.scope2_method.as_deref() == Some("market_based")).collect());
                    }
                    show_form.set(ShowForm::None);
                }
                Err(e) => error.set(e),
            }
        });
    };

    let add_market = move |_| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        error.set(String::new());
        let instr = mkt_instrument.get();
        let citation = if instr != "none" { Some(format!("Instrument: {instr}")) } else { None };
        let input = CreateSourceInput {
            entity_id: mkt_entity.get(),
            period_id: period.id,
            scope: 2,
            scope2_method: Some("market_based".into()),
            scope3_category: None,
            category_name: mkt_category.get(),
            ghg_type: "CO2".into(),
            activity_value: mkt_activity_value.get(),
            activity_unit: mkt_activity_unit.get(),
            activity_source: Some(mkt_activity_source.get()),
            emission_factor_value: mkt_ef_value.get(),
            emission_factor_unit: mkt_ef_unit.get(),
            emission_factor_source: mkt_ef_source.get(),
            emission_factor_citation: citation,
            gwp_value: 1.0,
            biogenic_co2_tco2e: None,
            uncertainty_pct: Some(mkt_uncertainty.get()),
            notes: { let n = mkt_notes.get(); if n.is_empty() { None } else { Some(n) } },
        };
        spawn_local(async move {
            match tauri::create_source(input).await {
                Ok(_) => {
                    if let Ok(all) = tauri::list_sources(period.id, Some(2)).await {
                        location_sources.set(all.iter().filter(|s| s.scope2_method.as_deref() == Some("location_based")).cloned().collect());
                        market_sources.set(all.into_iter().filter(|s| s.scope2_method.as_deref() == Some("market_based")).collect());
                    }
                    show_form.set(ShowForm::None);
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
                if let Ok(all) = tauri::list_sources(period.id, Some(2)).await {
                    location_sources.set(all.iter().filter(|s| s.scope2_method.as_deref() == Some("location_based")).cloned().collect());
                    market_sources.set(all.into_iter().filter(|s| s.scope2_method.as_deref() == Some("market_based")).collect());
                }
            });
        }
    };

    let input_cls = "w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none";
    let label_cls = "mb-1 block text-xs font-medium text-gray-400";

    view! {
        <div class="p-8">
            <div class="mb-6">
                <h1 class="text-xl font-bold text-gray-100">"Scope 2 — Energy Indirect Emissions"</h1>
                <p class="text-xs text-gray-500">"GRI 305-2 · ISO 14064-1 §5.3.2 · Both methods are mandatory"</p>
            </div>

            <div class="mb-6 rounded-xl border border-blue-800/50 bg-blue-950/20 p-4">
                <p class="text-xs font-semibold text-blue-400">"GRI 305-2 Dual-Method Requirement"</p>
                <p class="mt-1 text-xs text-blue-300/70">
                    "GRI 305-2 and the GHG Protocol require reporting BOTH location-based AND market-based figures. "
                    "Location-based uses grid-average emission factors. Market-based uses contractual instruments (RECs, PPAs) — "
                    "zero if no instruments held, residual mix factor otherwise."
                </p>
            </div>

            // Summary totals
            <div class="mb-6 grid gap-4 sm:grid-cols-2">
                <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
                    <div class="mb-2 flex items-center justify-between">
                        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">"Location-based"</p>
                        <button
                            on:click=move |_| show_form.update(|f| *f = if matches!(*f, ShowForm::Location) { ShowForm::None } else { ShowForm::Location })
                            class="rounded-lg bg-green-600 px-3 py-1 text-xs font-semibold text-white hover:bg-green-700"
                        >"+ Add"</button>
                    </div>
                    <p class="text-2xl font-bold text-gray-100">{move || fmt(total_tco2e(&location_sources.get()))}</p>
                    <p class="text-xs text-gray-500">
                        "tCO₂e · grid-average EF · "
                        {move || location_sources.get().len()}
                        {move || if location_sources.get().len() == 1 { " source" } else { " sources" }}
                    </p>
                </div>
                <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
                    <div class="mb-2 flex items-center justify-between">
                        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">"Market-based"</p>
                        <button
                            on:click=move |_| show_form.update(|f| *f = if matches!(*f, ShowForm::Market) { ShowForm::None } else { ShowForm::Market })
                            class="rounded-lg bg-green-600 px-3 py-1 text-xs font-semibold text-white hover:bg-green-700"
                        >"+ Add"</button>
                    </div>
                    <p class="text-2xl font-bold text-gray-100">{move || fmt(total_tco2e(&market_sources.get()))}</p>
                    <p class="text-xs text-gray-500">
                        "tCO₂e · contractual instruments · "
                        {move || market_sources.get().len()}
                        {move || if market_sources.get().len() == 1 { " source" } else { " sources" }}
                    </p>
                </div>
            </div>

            // Location-based form
            {move || matches!(show_form.get(), ShowForm::Location).then(|| view! {
                <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
                    <h3 class="mb-4 text-sm font-semibold text-gray-200">"Add Location-based Source"</h3>
                    <p class="mb-4 text-xs text-gray-500">"Use the grid-average emission factor for the region/country where energy is consumed."</p>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <div>
                            <label class=label_cls>"Category"</label>
                            <select prop:value=move || loc_category.get() on:change=move |ev| loc_category.set(event_target_value(&ev)) class=input_cls>
                                <option value="">"Select category…"</option>
                                {SCOPE2_CATEGORIES.iter().map(|c| view! { <option>{*c}</option> }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class=label_cls>"Activity value (kWh or MWh)"</label>
                            <input type="number" step="0.001"
                                prop:value=move || loc_activity_value.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { loc_activity_value.set(v); } }
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"Activity unit"</label>
                            <input type="text" placeholder="kWh, MWh…"
                                prop:value=move || loc_activity_unit.get()
                                on:input=move |ev| loc_activity_unit.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"Grid emission factor (kgCO₂e/kWh)"</label>
                            <input type="number" step="0.0001"
                                prop:value=move || loc_ef_value.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { loc_ef_value.set(v); } }
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"EF source (e.g. IEA, EPA eGRID, DEFRA)"</label>
                            <input type="text"
                                prop:value=move || loc_ef_source.get()
                                on:input=move |ev| loc_ef_source.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"Uncertainty (%)"</label>
                            <input type="number" step="1" min="0" max="100"
                                prop:value=move || loc_uncertainty.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { loc_uncertainty.set(v); } }
                                class=input_cls />
                        </div>
                        <div class="sm:col-span-2">
                            <label class=label_cls>"Notes"</label>
                            <input type="text"
                                prop:value=move || loc_notes.get()
                                on:input=move |ev| loc_notes.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                    </div>
                    {move || (!error.get().is_empty()).then(|| view! { <p class="mt-2 text-xs text-red-400">{error.get()}</p> })}
                    <div class="mt-4 flex gap-2">
                        <button on:click=add_location class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                        <button on:click=move |_| show_form.set(ShowForm::None) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">"Cancel"</button>
                    </div>
                </div>
            })}

            // Market-based form
            {move || matches!(show_form.get(), ShowForm::Market).then(|| view! {
                <div class="mb-6 rounded-xl border border-gray-700 bg-gray-900 p-5">
                    <h3 class="mb-4 text-sm font-semibold text-gray-200">"Add Market-based Source"</h3>
                    <p class="mb-4 text-xs text-gray-500">"Use zero if holding valid RECs/PPAs covering this consumption. Use residual mix EF if no instrument covers this source."</p>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <div>
                            <label class=label_cls>"Category"</label>
                            <select prop:value=move || mkt_category.get() on:change=move |ev| mkt_category.set(event_target_value(&ev)) class=input_cls>
                                <option value="">"Select category…"</option>
                                {SCOPE2_CATEGORIES.iter().map(|c| view! { <option>{*c}</option> }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class=label_cls>"Contractual instrument"</label>
                            <select prop:value=move || mkt_instrument.get() on:change=move |ev| mkt_instrument.set(event_target_value(&ev)) class=input_cls>
                                {INSTRUMENT_TYPES.iter().map(|(v, l)| view! { <option value=*v>{*l}</option> }).collect_view()}
                            </select>
                        </div>
                        <div>
                            <label class=label_cls>"Activity value"</label>
                            <input type="number" step="0.001"
                                prop:value=move || mkt_activity_value.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { mkt_activity_value.set(v); } }
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"Activity unit"</label>
                            <input type="text" placeholder="kWh, MWh…"
                                prop:value=move || mkt_activity_unit.get()
                                on:input=move |ev| mkt_activity_unit.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>
                                "Emission factor"
                                {move || {
                                    let i = mkt_instrument.get();
                                    (i == "REC" || i == "PPA" || i == "GG").then(|| view! {
                                        <span class="ml-1 text-green-600">" (0 if fully covered)"</span>
                                    })
                                }}
                            </label>
                            <input type="number" step="0.0001" min="0"
                                prop:value=move || mkt_ef_value.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { mkt_ef_value.set(v); } }
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"EF unit"</label>
                            <input type="text"
                                prop:value=move || mkt_ef_unit.get()
                                on:input=move |ev| mkt_ef_unit.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"EF source"</label>
                            <input type="text"
                                prop:value=move || mkt_ef_source.get()
                                on:input=move |ev| mkt_ef_source.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                        <div>
                            <label class=label_cls>"Uncertainty (%)"</label>
                            <input type="number" step="1" min="0" max="100"
                                prop:value=move || mkt_uncertainty.get()
                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { mkt_uncertainty.set(v); } }
                                class=input_cls />
                        </div>
                        <div class="sm:col-span-2">
                            <label class=label_cls>"Notes (instrument registry, vintage year, etc.)"</label>
                            <input type="text"
                                prop:value=move || mkt_notes.get()
                                on:input=move |ev| mkt_notes.set(event_target_value(&ev))
                                class=input_cls />
                        </div>
                    </div>
                    {move || (!error.get().is_empty()).then(|| view! { <p class="mt-2 text-xs text-red-400">{error.get()}</p> })}
                    <div class="mt-4 flex gap-2">
                        <button on:click=add_market class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                        <button on:click=move |_| show_form.set(ShowForm::None) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600">"Cancel"</button>
                    </div>
                </div>
            })}

            // Location-based table
            <div class="mb-6">
                <h2 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">"Location-based Sources"</h2>
                <div class="overflow-hidden rounded-xl border border-gray-800">
                    <table class="w-full text-sm">
                        <thead class="border-b border-gray-800 bg-gray-900/60">
                            <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
                                <th class="px-4 py-3">"Category"</th>
                                <th class="px-4 py-3">"Activity"</th>
                                <th class="px-4 py-3">"Grid EF"</th>
                                <th class="px-4 py-3">"tCO₂e"</th>
                                <th class="px-4 py-3">"±%"</th>
                                <th class="px-4 py-3"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800 bg-gray-900">
                            {move || {
                                let src = location_sources.get();
                                if src.is_empty() {
                                    view! {
                                        <tr><td colspan="6" class="px-4 py-6 text-center text-xs text-gray-500">"No location-based sources. Required for GRI 305-2."</td></tr>
                                    }.into_any()
                                } else {
                                    src.into_iter().map(|s| {
                                        let id = s.id;
                                        view! {
                                            <tr class="hover:bg-gray-800/40">
                                                <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
                                                <td class="px-4 py-3 text-gray-300">{s.activity_value}" "{s.activity_unit}</td>
                                                <td class="px-4 py-3 text-xs text-gray-500">{s.emission_factor_value}" "{s.emission_factor_unit}</td>
                                                <td class="px-4 py-3 font-semibold text-gray-200">{s.emissions_tco2e.map(|v| format!("{v:.3}")).unwrap_or_else(|| "—".into())}</td>
                                                <td class="px-4 py-3 text-xs text-gray-500">{s.uncertainty_pct.map(|u| format!("±{u}%")).unwrap_or_else(|| "—".into())}</td>
                                                <td class="px-4 py-3"><button on:click=move |_| remove(id) class="text-xs text-red-500 hover:text-red-400">"✕"</button></td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </div>

            // Market-based table
            <div>
                <h2 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">"Market-based Sources"</h2>
                <div class="overflow-hidden rounded-xl border border-gray-800">
                    <table class="w-full text-sm">
                        <thead class="border-b border-gray-800 bg-gray-900/60">
                            <tr class="text-left text-xs font-semibold uppercase tracking-wider text-gray-500">
                                <th class="px-4 py-3">"Category"</th>
                                <th class="px-4 py-3">"Activity"</th>
                                <th class="px-4 py-3">"EF (contractual)"</th>
                                <th class="px-4 py-3">"Instrument"</th>
                                <th class="px-4 py-3">"tCO₂e"</th>
                                <th class="px-4 py-3">"±%"</th>
                                <th class="px-4 py-3"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800 bg-gray-900">
                            {move || {
                                let src = market_sources.get();
                                if src.is_empty() {
                                    view! {
                                        <tr><td colspan="7" class="px-4 py-6 text-center text-xs text-gray-500">"No market-based sources. Required for GRI 305-2."</td></tr>
                                    }.into_any()
                                } else {
                                    src.into_iter().map(|s| {
                                        let id = s.id;
                                        view! {
                                            <tr class="hover:bg-gray-800/40">
                                                <td class="px-4 py-3 text-gray-200">{s.category_name}</td>
                                                <td class="px-4 py-3 text-gray-300">{s.activity_value}" "{s.activity_unit}</td>
                                                <td class="px-4 py-3 text-xs text-gray-500">{s.emission_factor_value}" "{s.emission_factor_unit}</td>
                                                <td class="px-4 py-3 text-xs text-gray-400">{s.emission_factor_citation.unwrap_or_else(|| "—".into())}</td>
                                                <td class="px-4 py-3 font-semibold text-gray-200">{s.emissions_tco2e.map(|v| format!("{v:.3}")).unwrap_or_else(|| "—".into())}</td>
                                                <td class="px-4 py-3 text-xs text-gray-500">{s.uncertainty_pct.map(|u| format!("±{u}%")).unwrap_or_else(|| "—".into())}</td>
                                                <td class="px-4 py-3"><button on:click=move |_| remove(id) class="text-xs text-red-500 hover:text-red-400">"✕"</button></td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
