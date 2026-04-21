use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::store::AppStore;
use crate::tauri;
use crate::types::{AirEntry, AuditEntry, Entity, IntensityResult, OdsEntry, Reduction, ReportingPeriod};

#[derive(Clone, PartialEq)]
enum Tab { Org, Periods, Entities, Supplemental, Audit, Enterprise }

#[component]
pub fn Settings() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let periods = RwSignal::new(Vec::<ReportingPeriod>::new());
    let entities = RwSignal::new(Vec::<Entity>::new());
    let audit_log = RwSignal::new(Vec::<AuditEntry>::new());
    let active_tab = RwSignal::new(Tab::Org);

    // Supplemental data
    let intensity_results = RwSignal::new(Vec::<IntensityResult>::new());
    let reductions = RwSignal::new(Vec::<Reduction>::new());
    let ods_entries = RwSignal::new(Vec::<OdsEntry>::new());
    let air_entries = RwSignal::new(Vec::<AirEntry>::new());

    // Org form
    let org_name = RwSignal::new(String::new());
    let org_boundary = RwSignal::new("operational_control".to_string());
    let org_base_year = RwSignal::new(2024i32);

    // Period form
    let new_period_year = RwSignal::new(2025i32);
    let new_period_gwp = RwSignal::new("AR6".to_string());
    let show_period_form = RwSignal::new(false);

    // Entity form
    let new_entity_name = RwSignal::new(String::new());
    let new_entity_type = RwSignal::new("subsidiary".to_string());
    let new_entity_ownership = RwSignal::new(100.0f64);
    let new_entity_country = RwSignal::new(String::new());
    let show_entity_form = RwSignal::new(false);

    // Intensity form
    let int_metric_name = RwSignal::new(String::new());
    let int_metric_value = RwSignal::new(0.0f64);
    let int_metric_unit = RwSignal::new(String::new());
    let int_scope1 = RwSignal::new(true);
    let int_scope2 = RwSignal::new(true);
    let int_scope3 = RwSignal::new(false);
    let show_int_form = RwSignal::new(false);

    // Reduction form
    let red_baseline_year = RwSignal::new(2024i32);
    let red_baseline_tco2e = RwSignal::new(0.0f64);
    let red_current_tco2e = RwSignal::new(0.0f64);
    let red_methodology = RwSignal::new(String::new());
    let show_red_form = RwSignal::new(false);

    // ODS form
    let ods_substance = RwSignal::new(String::new());
    let ods_production = RwSignal::new(0.0f64);
    let ods_imports = RwSignal::new(0.0f64);
    let ods_exports = RwSignal::new(0.0f64);
    let ods_cfc11 = RwSignal::new(0.0f64);
    let show_ods_form = RwSignal::new(false);

    // Air form
    let air_type = RwSignal::new("NOx".to_string());
    let air_substance = RwSignal::new(String::new());
    let air_value = RwSignal::new(0.0f64);
    let air_method = RwSignal::new("estimation".to_string());
    let show_air_form = RwSignal::new(false);

    let error = RwSignal::new(String::new());
    let success = RwSignal::new(String::new());

    Effect::new(move |_| {
        let org = store.active_org.get();
        if let Some(org) = org {
            org_name.set(org.name.clone());
            org_boundary.set(org.boundary_method.clone());
            org_base_year.set(org.base_year.unwrap_or(2024));
            let org_id = org.id;
            spawn_local(async move {
                if let Ok(ps) = tauri::list_periods(org_id).await { periods.set(ps); }
                if let Ok(ents) = tauri::list_entities(org_id).await { entities.set(ents); }
            });
        }
    });

    let load_supplemental = move || {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                if let Ok(v) = tauri::list_intensity_results(period.id).await { intensity_results.set(v); }
                if let Ok(v) = tauri::list_reductions(period.id).await { reductions.set(v); }
                if let Ok(v) = tauri::list_ods_emissions(period.id).await { ods_entries.set(v); }
                if let Ok(v) = tauri::list_air_emissions(period.id).await { air_entries.set(v); }
            });
        }
    };

    let save_org = move |_| {
        let org = store.active_org.get();
        if org.is_none() { return; }
        let org = org.unwrap();
        let name = org_name.get();
        let bm = org_boundary.get();
        let by = org_base_year.get();
        error.set(String::new()); success.set(String::new());
        spawn_local(async move {
            match tauri::update_org(org.id, &name, &bm, Some(by)).await {
                Ok(_) => {
                    store.active_org.update(|o| {
                        if let Some(o) = o {
                            o.name = name;
                            o.boundary_method = bm;
                            o.base_year = Some(by);
                        }
                    });
                    success.set("Organisation updated.".into());
                }
                Err(e) => error.set(e),
            }
        });
    };

    let add_period = move |_| {
        let org = store.active_org.get();
        if org.is_none() { return; }
        let org = org.unwrap();
        let year = new_period_year.get();
        let gwp = new_period_gwp.get();
        error.set(String::new()); success.set(String::new());
        spawn_local(async move {
            match tauri::create_period(org.id, year, &gwp).await {
                Ok(p) => {
                    periods.update(|ps| ps.push(p));
                    show_period_form.set(false);
                    success.set(format!("Period {year} created."));
                }
                Err(e) => error.set(e),
            }
        });
    };

    let add_entity = move |_| {
        let org = store.active_org.get();
        if org.is_none() { return; }
        let org = org.unwrap();
        let name = new_entity_name.get();
        let etype = new_entity_type.get();
        let ownership = new_entity_ownership.get();
        let country = new_entity_country.get();
        error.set(String::new()); success.set(String::new());
        spawn_local(async move {
            match tauri::create_entity(
                org.id, &name, &etype,
                Some(ownership), true, true,
                if country.is_empty() { None } else { Some(country) },
            ).await {
                Ok(e) => {
                    entities.update(|es| es.push(e));
                    show_entity_form.set(false);
                    new_entity_name.set(String::new());
                    success.set("Entity added.".into());
                }
                Err(e) => error.set(e),
            }
        });
    };

    let save_intensity = move |_| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        let name = int_metric_name.get();
        if name.trim().is_empty() || int_metric_value.get() <= 0.0 { return; }
        error.set(String::new());
        spawn_local(async move {
            match tauri::save_intensity_metric(
                period.id, name.trim(), int_metric_value.get(),
                &int_metric_unit.get(), int_scope1.get(), int_scope2.get(), int_scope3.get(),
            ).await {
                Ok(_) => {
                    if let Ok(v) = tauri::list_intensity_results(period.id).await { intensity_results.set(v); }
                    show_int_form.set(false);
                    int_metric_name.set(String::new());
                    int_metric_value.set(0.0);
                    int_metric_unit.set(String::new());
                }
                Err(e) => error.set(e),
            }
        });
    };

    let remove_intensity = move |name: String| {
        let period = store.active_period.get();
        if let Some(period) = period {
            spawn_local(async move {
                let _ = tauri::delete_intensity_result(period.id, &name).await;
                if let Ok(v) = tauri::list_intensity_results(period.id).await { intensity_results.set(v); }
            });
        }
    };

    let save_reduction = move |_| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        let method = red_methodology.get();
        if method.trim().is_empty() { return; }
        error.set(String::new());
        spawn_local(async move {
            match tauri::create_reduction(
                period.id, red_baseline_year.get(),
                red_baseline_tco2e.get(), red_current_tco2e.get(), method.trim(),
            ).await {
                Ok(_) => {
                    if let Ok(v) = tauri::list_reductions(period.id).await { reductions.set(v); }
                    show_red_form.set(false);
                    red_methodology.set(String::new());
                }
                Err(e) => error.set(e),
            }
        });
    };

    let remove_reduction = move |id: i64| {
        let period = store.active_period.get();
        spawn_local(async move {
            let _ = tauri::delete_reduction(id).await;
            if let Some(p) = period {
                if let Ok(v) = tauri::list_reductions(p.id).await { reductions.set(v); }
            }
        });
    };

    let save_ods = move |_| {
        let period = store.active_period.get();
        if period.is_none() { return; }
        let period = period.unwrap();
        let subst = ods_substance.get();
        if subst.trim().is_empty() { return; }
        error.set(String::new());
        spawn_local(async move {
            match tauri::create_ods_emission(
                period.id, subst.trim(),
                ods_production.get(), ods_imports.get(), ods_exports.get(), ods_cfc11.get(),
            ).await {
                Ok(_) => {
                    if let Ok(v) = tauri::list_ods_emissions(period.id).await { ods_entries.set(v); }
                    show_ods_form.set(false);
                    ods_substance.set(String::new());
                }
                Err(e) => error.set(e),
            }
        });
    };

    let remove_ods = move |id: i64| {
        let period = store.active_period.get();
        spawn_local(async move {
            let _ = tauri::delete_ods_emission(id).await;
            if let Some(p) = period {
                if let Ok(v) = tauri::list_ods_emissions(p.id).await { ods_entries.set(v); }
            }
        });
    };

    let save_air = move |_| {
        let period = store.active_period.get();
        if period.is_none() || air_value.get() <= 0.0 { return; }
        let period = period.unwrap();
        let etype = air_type.get();
        let subst = air_substance.get();
        let val = air_value.get();
        let method = air_method.get();
        error.set(String::new());
        spawn_local(async move {
            match tauri::create_air_emission(
                period.id, &etype,
                if subst.is_empty() { None } else { Some(subst) },
                val, &method,
            ).await {
                Ok(_) => {
                    if let Ok(v) = tauri::list_air_emissions(period.id).await { air_entries.set(v); }
                    show_air_form.set(false);
                    air_substance.set(String::new());
                    air_value.set(0.0);
                }
                Err(e) => error.set(e),
            }
        });
    };

    let remove_air = move |id: i64| {
        let period = store.active_period.get();
        spawn_local(async move {
            let _ = tauri::delete_air_emission(id).await;
            if let Some(p) = period {
                if let Ok(v) = tauri::list_air_emissions(p.id).await { air_entries.set(v); }
            }
        });
    };

    let load_audit = move |_| {
        let org = store.active_org.get();
        if let Some(org) = org {
            spawn_local(async move {
                if let Ok(log) = tauri::get_audit_log("organizations", org.id).await {
                    audit_log.set(log);
                }
            });
        }
    };

    let ic = "w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none";
    let lc = "mb-1 block text-xs font-medium text-gray-400";

    let tab_btn = move |t: Tab, label: &'static str| {
        let t_clone = t.clone();
        view! {
            <button
                on:click=move |_| {
                    if t_clone == Tab::Supplemental { load_supplemental(); }
                    active_tab.set(t_clone.clone());
                }
                class=move || if active_tab.get() == t.clone() {
                    "rounded-lg px-4 py-2 text-sm font-medium transition-colors bg-green-600 text-white"
                } else {
                    "rounded-lg px-4 py-2 text-sm font-medium transition-colors text-gray-500 hover:text-gray-300"
                }
            >{label}</button>
        }
    };

    view! {
        <div class="p-8">
            <div class="mb-6">
                <h1 class="text-xl font-bold text-gray-100">"Settings"</h1>
                <p class="text-xs text-gray-500">"Organisation configuration, periods, entities, and audit trail"</p>
            </div>

            // Tabs
            <div class="mb-6 flex flex-wrap gap-1 rounded-xl border border-gray-800 bg-gray-900 p-1">
                {tab_btn(Tab::Org, "Organisation")}
                {tab_btn(Tab::Periods, "Periods")}
                {tab_btn(Tab::Entities, "Entities")}
                {tab_btn(Tab::Supplemental, "Supplemental")}
                {tab_btn(Tab::Audit, "Audit Trail")}
                {tab_btn(Tab::Enterprise, "Enterprise")}
            </div>

            {move || (!error.get().is_empty()).then(|| view! {
                <div class="mb-4 rounded-lg border border-red-800 bg-red-950/20 p-3 text-xs text-red-400">{error.get()}</div>
            })}
            {move || (!success.get().is_empty()).then(|| view! {
                <div class="mb-4 rounded-lg border border-green-800 bg-green-950/20 p-3 text-xs text-green-400">{success.get()}</div>
            })}

            {move || match active_tab.get() {
                Tab::Org => view! {
                    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                        <h2 class="mb-4 text-sm font-semibold text-gray-200">"Organisation Details"</h2>
                        <div class="space-y-4">
                            <div>
                                <label class=lc>"Organisation name"</label>
                                <input type="text" prop:value=move || org_name.get()
                                    on:input=move |ev| org_name.set(event_target_value(&ev)) class=ic />
                            </div>
                            <div>
                                <label class=lc>"Organisational boundary method "<span class="text-gray-600">"(ISO 14064-1 §5.2)"</span></label>
                                <div class="space-y-2">
                                    {[
                                        ("operational_control", "Operational Control"),
                                        ("financial_control", "Financial Control"),
                                        ("equity_share", "Equity Share"),
                                    ].iter().map(|(val, label)| {
                                        let val = *val;
                                        view! {
                                            <label class=move || if org_boundary.get() == val {
                                                "flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors border-green-700 bg-green-950/20"
                                            } else {
                                                "flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors border-gray-800 hover:border-gray-700"
                                            }>
                                                <input type="radio"
                                                    prop:checked=move || org_boundary.get() == val
                                                    on:change=move |_| org_boundary.set(val.to_string()) />
                                                <span class="text-sm text-gray-200">{*label}</span>
                                            </label>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                            <div>
                                <label class=lc>"Base year"</label>
                                <input type="number" min="2000" max="2030"
                                    prop:value=move || org_base_year.get()
                                    on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { org_base_year.set(v); } }
                                    class="w-32 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 focus:border-green-600 focus:outline-none" />
                            </div>
                        </div>
                        <button on:click=save_org
                            class="mt-5 rounded-lg bg-green-600 px-5 py-2 text-sm font-semibold text-white hover:bg-green-700">
                            "Save changes"
                        </button>
                    </div>
                }.into_any(),

                Tab::Periods => view! {
                    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                        <div class="mb-4 flex items-center justify-between">
                            <h2 class="text-sm font-semibold text-gray-200">"Reporting Periods"</h2>
                            <button on:click=move |_| show_period_form.update(|v| *v = !*v)
                                class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                "+ New period"
                            </button>
                        </div>
                        {move || show_period_form.get().then(|| view! {
                            <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <div>
                                        <label class=lc>"Year"</label>
                                        <input type="number" min="2000" max="2030"
                                            prop:value=move || new_period_year.get()
                                            on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { new_period_year.set(v); } }
                                            class=ic />
                                    </div>
                                    <div>
                                        <label class=lc>"GWP version"</label>
                                        <select prop:value=move || new_period_gwp.get() on:change=move |ev| new_period_gwp.set(event_target_value(&ev)) class=ic>
                                            <option value="AR6">"AR6 (2021) — Recommended"</option>
                                            <option value="AR5">"AR5 (2013)"</option>
                                            <option value="AR4">"AR4 (2007)"</option>
                                        </select>
                                    </div>
                                </div>
                                <div class="mt-3 flex gap-2">
                                    <button on:click=add_period class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Create"</button>
                                    <button on:click=move |_| show_period_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                </div>
                            </div>
                        })}
                        <div class="space-y-2">
                            {move || {
                                let ps = periods.get();
                                if ps.is_empty() {
                                    view! { <p class="text-sm text-gray-500">"No periods yet."</p> }.into_any()
                                } else {
                                    ps.into_iter().map(|p| {
                                        let p_id = p.id;
                                        let p_year = p.year;
                                        let p_clone = p.clone();
                                        view! {
                                            <div class=move || {
                                                let active = store.active_period.get().map(|ap| ap.id == p_id).unwrap_or(false);
                                                if active { "flex items-center justify-between rounded-lg border border-green-800/50 bg-green-950/10 px-4 py-3" }
                                                else { "flex items-center justify-between rounded-lg border border-gray-800 px-4 py-3" }
                                            }>
                                                <div>
                                                    <p class="text-sm font-medium text-gray-200">{p.year}" · "{p.gwp_ar_version}</p>
                                                    <p class="text-xs text-gray-500">{p.start_date}" → "{p.end_date}" · "{p.status}</p>
                                                </div>
                                                {move || {
                                                    if store.active_period.get().map(|ap| ap.id == p_id).unwrap_or(false) {
                                                        view! { <span class="text-xs text-green-500">"Active"</span> }.into_any()
                                                    } else {
                                                        let p2 = p_clone.clone();
                                                        view! {
                                                            <button
                                                                on:click=move |_| {
                                                                    store.active_period.set(Some(p2.clone()));
                                                                    success.set(format!("Active period set to {p_year}."));
                                                                }
                                                                class="rounded-lg border border-gray-700 px-3 py-1 text-xs text-gray-400 hover:border-gray-600">
                                                                "Set active"
                                                            </button>
                                                        }.into_any()
                                                    }
                                                }}
                                            </div>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any(),

                Tab::Entities => view! {
                    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                        <div class="mb-4 flex items-center justify-between">
                            <h2 class="text-sm font-semibold text-gray-200">"Legal Entities / Facilities"</h2>
                            <button on:click=move |_| show_entity_form.update(|v| *v = !*v)
                                class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                "+ Add entity"
                            </button>
                        </div>
                        {move || show_entity_form.get().then(|| view! {
                            <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <div>
                                        <label class=lc>"Name"</label>
                                        <input type="text" placeholder="Acme Corp UK Ltd"
                                            prop:value=move || new_entity_name.get()
                                            on:input=move |ev| new_entity_name.set(event_target_value(&ev))
                                            class=ic />
                                    </div>
                                    <div>
                                        <label class=lc>"Type"</label>
                                        <select prop:value=move || new_entity_type.get() on:change=move |ev| new_entity_type.set(event_target_value(&ev)) class=ic>
                                            {["parent","subsidiary","facility","jv","branch"].iter().map(|t| view! { <option>{*t}</option> }).collect_view()}
                                        </select>
                                    </div>
                                    <div>
                                        <label class=lc>"Ownership % (for equity share)"</label>
                                        <input type="number" min="0" max="100" step="0.1"
                                            prop:value=move || new_entity_ownership.get()
                                            on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { new_entity_ownership.set(v); } }
                                            class=ic />
                                    </div>
                                    <div>
                                        <label class=lc>"Country code"</label>
                                        <input type="text" placeholder="GB, US, AU…" maxlength="2"
                                            prop:value=move || new_entity_country.get()
                                            on:input=move |ev| new_entity_country.set(event_target_value(&ev))
                                            class=ic />
                                    </div>
                                </div>
                                <div class="mt-3 flex gap-2">
                                    <button on:click=add_entity class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Add"</button>
                                    <button on:click=move |_| show_entity_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                </div>
                            </div>
                        })}
                        <div class="space-y-2">
                            {move || {
                                let ents = entities.get();
                                if ents.is_empty() {
                                    view! { <p class="text-sm text-gray-500">"No entities yet."</p> }.into_any()
                                } else {
                                    ents.into_iter().map(|e| view! {
                                        <div class="flex items-center justify-between rounded-lg border border-gray-800 px-4 py-3">
                                            <div>
                                                <p class="text-sm font-medium text-gray-200">{e.name}</p>
                                                <p class="text-xs text-gray-500">
                                                    {e.entity_type.clone()}
                                                    {e.ownership_pct.map(|p| format!(" · {p}% owned")).unwrap_or_default()}
                                                    {e.country_code.map(|c| format!(" · {c}")).unwrap_or_default()}
                                                </p>
                                            </div>
                                            <span class="rounded-full border border-gray-700 px-2 py-0.5 text-[10px] text-gray-500">{e.entity_type}</span>
                                        </div>
                                    }).collect_view().into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any(),

                Tab::Supplemental => view! {
                    <div class="space-y-6">
                        // Intensity (305-4)
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <div class="mb-4 flex items-center justify-between">
                                <div>
                                    <h2 class="text-sm font-semibold text-gray-200">"Emissions Intensity (GRI 305-4)"</h2>
                                    <p class="text-xs text-gray-500">"tCO₂e per unit of an activity metric"</p>
                                </div>
                                <button on:click=move |_| show_int_form.update(|v| *v = !*v)
                                    class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                    "+ Add metric"
                                </button>
                            </div>
                            {move || show_int_form.get().then(|| view! {
                                <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                    <div class="grid gap-3 sm:grid-cols-2">
                                        <div class="sm:col-span-2">
                                            <label class=lc>"Metric name (e.g. Revenue USD, Units Produced)"</label>
                                            <input type="text" placeholder="Revenue (USD M)"
                                                prop:value=move || int_metric_name.get()
                                                on:input=move |ev| int_metric_name.set(event_target_value(&ev))
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Metric value"</label>
                                            <input type="number" step="0.01" min="0"
                                                prop:value=move || int_metric_value.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { int_metric_value.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Unit"</label>
                                            <input type="text" placeholder="USD M, units, FTE…"
                                                prop:value=move || int_metric_unit.get()
                                                on:input=move |ev| int_metric_unit.set(event_target_value(&ev))
                                                class=ic />
                                        </div>
                                    </div>
                                    <div class="mt-3 flex flex-wrap gap-4">
                                        <label class="flex items-center gap-2 text-xs text-gray-400">
                                            <input type="checkbox"
                                                prop:checked=move || int_scope1.get()
                                                on:change=move |ev| {
                                                    let el: web_sys::HtmlInputElement = event_target(&ev);
                                                    int_scope1.set(el.checked());
                                                } />
                                            "Include Scope 1"
                                        </label>
                                        <label class="flex items-center gap-2 text-xs text-gray-400">
                                            <input type="checkbox"
                                                prop:checked=move || int_scope2.get()
                                                on:change=move |ev| {
                                                    let el: web_sys::HtmlInputElement = event_target(&ev);
                                                    int_scope2.set(el.checked());
                                                } />
                                            "Include Scope 2"
                                        </label>
                                        <label class="flex items-center gap-2 text-xs text-gray-400">
                                            <input type="checkbox"
                                                prop:checked=move || int_scope3.get()
                                                on:change=move |ev| {
                                                    let el: web_sys::HtmlInputElement = event_target(&ev);
                                                    int_scope3.set(el.checked());
                                                } />
                                            "Include Scope 3"
                                        </label>
                                    </div>
                                    <div class="mt-3 flex gap-2">
                                        <button on:click=save_intensity class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                                        <button on:click=move |_| show_int_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                    </div>
                                </div>
                            })}
                            {move || {
                                let ir = intensity_results.get();
                                if ir.is_empty() {
                                    view! { <p class="text-xs text-gray-500">"No intensity metrics defined yet."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-hidden rounded-lg border border-gray-800">
                                            <table class="w-full text-xs">
                                                <thead class="border-b border-gray-800 bg-gray-800/40">
                                                    <tr class="text-left text-gray-500">
                                                        <th class="px-3 py-2">"Metric"</th>
                                                        <th class="px-3 py-2">"Value"</th>
                                                        <th class="px-3 py-2">"Ratio (tCO₂e/unit)"</th>
                                                        <th class="px-3 py-2">"Scopes"</th>
                                                        <th class="px-3 py-2"></th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-800">
                                                    {ir.into_iter().map(|r| {
                                                        let name = r.metric_name.clone();
                                                        let scopes = [
                                                            r.includes_scope1.then_some("S1"),
                                                            r.includes_scope2.then_some("S2"),
                                                            r.includes_scope3.then_some("S3"),
                                                        ].into_iter().flatten().collect::<Vec<_>>().join("+");
                                                        view! {
                                                            <tr class="text-gray-300">
                                                                <td class="px-3 py-2">{r.metric_name}</td>
                                                                <td class="px-3 py-2">{r.metric_value}" "{r.metric_unit}</td>
                                                                <td class="px-3 py-2 font-semibold">{format!("{:.4}", r.intensity_ratio)}</td>
                                                                <td class="px-3 py-2 text-gray-500">{scopes}</td>
                                                                <td class="px-3 py-2">
                                                                    <button on:click=move |_| remove_intensity(name.clone()) class="text-red-500 hover:text-red-400">"✕"</button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // Reductions (305-5)
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <div class="mb-4 flex items-center justify-between">
                                <div>
                                    <h2 class="text-sm font-semibold text-gray-200">"Emissions Reductions (GRI 305-5)"</h2>
                                    <p class="text-xs text-gray-500">"Reductions from specific initiatives vs baseline."</p>
                                </div>
                                <button on:click=move |_| show_red_form.update(|v| *v = !*v)
                                    class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                    "+ Add"
                                </button>
                            </div>
                            {move || show_red_form.get().then(|| view! {
                                <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                    <div class="grid gap-3 sm:grid-cols-2">
                                        <div>
                                            <label class=lc>"Baseline year"</label>
                                            <input type="number" min="2000" max="2030"
                                                prop:value=move || red_baseline_year.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { red_baseline_year.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Baseline emissions (tCO₂e)"</label>
                                            <input type="number" step="0.01" min="0"
                                                prop:value=move || red_baseline_tco2e.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { red_baseline_tco2e.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Current period emissions (tCO₂e)"</label>
                                            <input type="number" step="0.01" min="0"
                                                prop:value=move || red_current_tco2e.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { red_current_tco2e.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Methodology"</label>
                                            <input type="text" placeholder="e.g. Building electrification, LED retrofit"
                                                prop:value=move || red_methodology.get()
                                                on:input=move |ev| red_methodology.set(event_target_value(&ev))
                                                class=ic />
                                        </div>
                                    </div>
                                    <div class="mt-3 flex gap-2">
                                        <button on:click=save_reduction class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                                        <button on:click=move |_| show_red_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                    </div>
                                </div>
                            })}
                            {move || {
                                let reds = reductions.get();
                                if reds.is_empty() {
                                    view! { <p class="text-xs text-gray-500">"No reductions recorded yet."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-hidden rounded-lg border border-gray-800">
                                            <table class="w-full text-xs">
                                                <thead class="border-b border-gray-800 bg-gray-800/40">
                                                    <tr class="text-left text-gray-500">
                                                        <th class="px-3 py-2">"Baseline year"</th>
                                                        <th class="px-3 py-2">"Baseline (tCO₂e)"</th>
                                                        <th class="px-3 py-2">"Current (tCO₂e)"</th>
                                                        <th class="px-3 py-2">"Reduction"</th>
                                                        <th class="px-3 py-2">"Methodology"</th>
                                                        <th class="px-3 py-2"></th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-800">
                                                    {reds.into_iter().map(|r| {
                                                        let id = r.id;
                                                        view! {
                                                            <tr class="text-gray-300">
                                                                <td class="px-3 py-2">{r.baseline_year}</td>
                                                                <td class="px-3 py-2">{format!("{:.2}", r.baseline_tco2e)}</td>
                                                                <td class="px-3 py-2">{format!("{:.2}", r.current_tco2e)}</td>
                                                                <td class="px-3 py-2 font-semibold text-green-400">
                                                                    {format!("{:.2} tCO₂e ({:.1}%)", r.reduction_tco2e, r.reduction_pct)}
                                                                </td>
                                                                <td class="px-3 py-2 text-gray-500">{r.methodology}</td>
                                                                <td class="px-3 py-2"><button on:click=move |_| remove_reduction(id) class="text-red-500 hover:text-red-400">"✕"</button></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // ODS (305-6)
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <div class="mb-4 flex items-center justify-between">
                                <div>
                                    <h2 class="text-sm font-semibold text-gray-200">"Ozone-Depleting Substances (GRI 305-6)"</h2>
                                    <p class="text-xs text-gray-500">"Production, imports, and exports in metric tonnes and CFC-11 equivalent."</p>
                                </div>
                                <button on:click=move |_| show_ods_form.update(|v| *v = !*v)
                                    class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                    "+ Add"
                                </button>
                            </div>
                            {move || show_ods_form.get().then(|| view! {
                                <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                    <div class="grid gap-3 sm:grid-cols-2">
                                        <div class="sm:col-span-2">
                                            <label class=lc>"Substance (e.g. R-22, R-410A, Halon-1301)"</label>
                                            <input type="text"
                                                prop:value=move || ods_substance.get()
                                                on:input=move |ev| ods_substance.set(event_target_value(&ev))
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Production (metric tonnes)"</label>
                                            <input type="number" step="0.001" min="0"
                                                prop:value=move || ods_production.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { ods_production.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Imports (metric tonnes)"</label>
                                            <input type="number" step="0.001" min="0"
                                                prop:value=move || ods_imports.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { ods_imports.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Exports (metric tonnes)"</label>
                                            <input type="number" step="0.001" min="0"
                                                prop:value=move || ods_exports.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { ods_exports.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"CFC-11 equivalent (metric tonnes)"</label>
                                            <input type="number" step="0.001" min="0"
                                                prop:value=move || ods_cfc11.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { ods_cfc11.set(v); } }
                                                class=ic />
                                        </div>
                                    </div>
                                    <div class="mt-3 flex gap-2">
                                        <button on:click=save_ods class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                                        <button on:click=move |_| show_ods_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                    </div>
                                </div>
                            })}
                            {move || {
                                let ods = ods_entries.get();
                                if ods.is_empty() {
                                    view! { <p class="text-xs text-gray-500">"No ODS entries recorded yet."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-hidden rounded-lg border border-gray-800">
                                            <table class="w-full text-xs">
                                                <thead class="border-b border-gray-800 bg-gray-800/40">
                                                    <tr class="text-left text-gray-500">
                                                        <th class="px-3 py-2">"Substance"</th>
                                                        <th class="px-3 py-2">"Production (t)"</th>
                                                        <th class="px-3 py-2">"Imports (t)"</th>
                                                        <th class="px-3 py-2">"Exports (t)"</th>
                                                        <th class="px-3 py-2">"CFC-11 eq (t)"</th>
                                                        <th class="px-3 py-2"></th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-800">
                                                    {ods.into_iter().map(|e| {
                                                        let id = e.id;
                                                        view! {
                                                            <tr class="text-gray-300">
                                                                <td class="px-3 py-2 font-medium">{e.substance}</td>
                                                                <td class="px-3 py-2">{e.production_metric_tons}</td>
                                                                <td class="px-3 py-2">{e.imports_metric_tons}</td>
                                                                <td class="px-3 py-2">{e.exports_metric_tons}</td>
                                                                <td class="px-3 py-2">{e.cfc11_equivalent}</td>
                                                                <td class="px-3 py-2"><button on:click=move |_| remove_ods(id) class="text-red-500 hover:text-red-400">"✕"</button></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // Air emissions (305-7)
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <div class="mb-4 flex items-center justify-between">
                                <div>
                                    <h2 class="text-sm font-semibold text-gray-200">"Air Emissions (GRI 305-7)"</h2>
                                    <p class="text-xs text-gray-500">"NOx, SOx, VOC, particulate matter in metric tonnes."</p>
                                </div>
                                <button on:click=move |_| show_air_form.update(|v| *v = !*v)
                                    class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-green-700">
                                    "+ Add"
                                </button>
                            </div>
                            {move || show_air_form.get().then(|| view! {
                                <div class="mb-5 rounded-lg border border-gray-700 p-4">
                                    <div class="grid gap-3 sm:grid-cols-2">
                                        <div>
                                            <label class=lc>"Emission type"</label>
                                            <select prop:value=move || air_type.get() on:change=move |ev| air_type.set(event_target_value(&ev)) class=ic>
                                                {["NOx","SOx","VOC","PM","HAP","other"].iter().map(|t| view! { <option>{*t}</option> }).collect_view()}
                                            </select>
                                        </div>
                                        <div>
                                            <label class=lc>"Substance (optional, e.g. SO₂, PM2.5)"</label>
                                            <input type="text" placeholder="Optional"
                                                prop:value=move || air_substance.get()
                                                on:input=move |ev| air_substance.set(event_target_value(&ev))
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Value (metric tonnes)"</label>
                                            <input type="number" step="0.001" min="0"
                                                prop:value=move || air_value.get()
                                                on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<f64>() { air_value.set(v); } }
                                                class=ic />
                                        </div>
                                        <div>
                                            <label class=lc>"Measurement method"</label>
                                            <select prop:value=move || air_method.get() on:change=move |ev| air_method.set(event_target_value(&ev)) class=ic>
                                                <option value="direct_measurement">"Direct measurement"</option>
                                                <option value="estimation">"Estimation"</option>
                                                <option value="balance">"Mass balance"</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div class="mt-3 flex gap-2">
                                        <button on:click=save_air class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">"Save"</button>
                                        <button on:click=move |_| show_air_form.set(false) class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                    </div>
                                </div>
                            })}
                            {move || {
                                let air = air_entries.get();
                                if air.is_empty() {
                                    view! { <p class="text-xs text-gray-500">"No air emissions recorded yet."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-hidden rounded-lg border border-gray-800">
                                            <table class="w-full text-xs">
                                                <thead class="border-b border-gray-800 bg-gray-800/40">
                                                    <tr class="text-left text-gray-500">
                                                        <th class="px-3 py-2">"Type"</th>
                                                        <th class="px-3 py-2">"Substance"</th>
                                                        <th class="px-3 py-2">"Value (t)"</th>
                                                        <th class="px-3 py-2">"Method"</th>
                                                        <th class="px-3 py-2"></th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-800">
                                                    {air.into_iter().map(|e| {
                                                        let id = e.id;
                                                        view! {
                                                            <tr class="text-gray-300">
                                                                <td class="px-3 py-2 font-medium">{e.emission_type}</td>
                                                                <td class="px-3 py-2 text-gray-500">{e.substance.unwrap_or_else(|| "—".into())}</td>
                                                                <td class="px-3 py-2">{e.value_metric_tons}</td>
                                                                <td class="px-3 py-2 text-gray-500">{e.measurement_method.replace('_', " ")}</td>
                                                                <td class="px-3 py-2"><button on:click=move |_| remove_air(id) class="text-red-500 hover:text-red-400">"✕"</button></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any(),

                Tab::Audit => view! {
                    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                        <div class="mb-4 flex items-center justify-between">
                            <div>
                                <h2 class="text-sm font-semibold text-gray-200">"Immutable Audit Trail"</h2>
                                <p class="text-xs text-gray-500">"ISO 14064-1 §5.5 · All changes are logged and cannot be deleted"</p>
                            </div>
                            <button on:click=load_audit
                                class="rounded-lg border border-gray-700 px-3 py-1.5 text-xs text-gray-400 hover:border-gray-600">
                                "Load log"
                            </button>
                        </div>
                        {move || {
                            let log = audit_log.get();
                            if log.is_empty() {
                                view! { <p class="text-sm text-gray-500">"Click \"Load log\" to view the audit trail."</p> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-hidden rounded-lg border border-gray-800">
                                        <table class="w-full text-xs">
                                            <thead class="border-b border-gray-800 bg-gray-800/40">
                                                <tr class="text-left text-gray-500">
                                                    <th class="px-3 py-2">"Timestamp"</th>
                                                    <th class="px-3 py-2">"Table"</th>
                                                    <th class="px-3 py-2">"Action"</th>
                                                    <th class="px-3 py-2">"Field"</th>
                                                    <th class="px-3 py-2">"Old"</th>
                                                    <th class="px-3 py-2">"New"</th>
                                                    <th class="px-3 py-2">"Reason"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-800">
                                                {log.into_iter().map(|entry| {
                                                    let ts = entry.get("timestamp")
                                                        .and_then(|v| v.as_i64())
                                                        .map(|t| {
                                                            let dt = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((t * 1000) as f64));
                                                            dt.to_locale_string("en-US", &wasm_bindgen::JsValue::UNDEFINED).as_string().unwrap_or_default()
                                                        })
                                                        .unwrap_or_else(|| "—".into());
                                                    let table = entry.get("table_name").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let field = entry.get("field_name").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let old = entry.get("old_value").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let new = entry.get("new_value").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                    let action_color = if action == "DELETE" { "text-red-400" }
                                                        else if action == "UPDATE" { "text-yellow-400" }
                                                        else { "text-green-400" };
                                                    view! {
                                                        <tr class="text-gray-400 hover:bg-gray-800/20">
                                                            <td class="px-3 py-2 font-mono">{ts}</td>
                                                            <td class="px-3 py-2">{table}</td>
                                                            <td class=format!("px-3 py-2 {action_color}")>{action}</td>
                                                            <td class="px-3 py-2">{field}</td>
                                                            <td class="px-3 py-2 text-gray-600">{old}</td>
                                                            <td class="px-3 py-2">{new}</td>
                                                            <td class="px-3 py-2 text-gray-500">{reason}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                }.into_any(),

                Tab::Enterprise => view! {
                    <div class="space-y-4">
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <h2 class="mb-1 text-sm font-semibold text-gray-200">"Enterprise Access"</h2>
                            <p class="mb-5 text-xs text-gray-500">"Cloud sync, multi-user access, SSO, and priority support."</p>
                            <div class="rounded-lg border border-green-800/50 bg-green-950/20 p-4">
                                <div class="mb-3">
                                    <p class="text-sm font-semibold text-green-400">"14-day free trial — no credit card required"</p>
                                    <p class="mt-1 text-xs text-gray-400">
                                        "Try all Enterprise features: cloud sync, 5 team seats, and SSO. "
                                        "Your local data stays on your device throughout."
                                    </p>
                                </div>
                                <button class="rounded-lg bg-green-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-green-700">
                                    "Start free trial →"
                                </button>
                                <p class="mt-2 text-[10px] text-gray-600">
                                    "Opens browser for SSO login. After login, trial activates immediately. "
                                    "Upgrade to paid at any time — $20/seat/month."
                                </p>
                            </div>
                            <div class="mt-4 rounded-lg border border-gray-800 p-4">
                                <div class="mb-2 flex items-center justify-between">
                                    <p class="text-sm font-medium text-gray-300">"Already have a licence?"</p>
                                    <span class="rounded-full border border-gray-700 px-2 py-0.5 text-[10px] text-gray-500">"Not connected"</span>
                                </div>
                                <button class="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:border-gray-600 hover:text-gray-200">
                                    "Connect to team →"
                                </button>
                            </div>
                        </div>
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <p class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">"Included in Enterprise & Trial"</p>
                            <div class="space-y-2">
                                {[
                                    "Multi-user access — admin, editor, and viewer roles",
                                    "Invite team members by email, manage seats",
                                    "Cloud sync — real-time, across all your devices",
                                    "Single sign-on — Okta, Azure AD, Google Workspace",
                                    "Priority support with SLA",
                                    "14-day free trial · 5 seats · no card required",
                                ].iter().map(|f| view! {
                                    <div class="flex items-start gap-2 text-xs">
                                        <span class="mt-0.5 text-green-500">"✓"</span>
                                        <span class="text-gray-400">{*f}</span>
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>
                        <p class="text-xs text-gray-600">
                            "Need a custom quote or on-premise deployment? "
                            <a href="https://c22.space/hire" target="_blank" rel="noopener"
                                class="text-gray-400 underline hover:text-gray-200">
                                "Contact c22 →"
                            </a>
                        </p>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
