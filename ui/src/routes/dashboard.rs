use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::store::AppStore;
use crate::tauri;
use crate::types::PeriodInventory;

fn fmt(n: Option<f64>) -> String {
    n.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into())
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let inventory = RwSignal::new(None::<PeriodInventory>);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(String::new());

    Effect::new(move |_| {
        let period = store.active_period.get();
        if let Some(period) = period {
            loading.set(true);
            error.set(String::new());
            spawn_local(async move {
                match tauri::calculate_period(period.id).await {
                    Ok(inv) => inventory.set(Some(inv)),
                    Err(e) => error.set(e),
                }
                loading.set(false);
            });
        }
    });

    view! {
        <div class="p-8">
            <div class="mb-6">
                <h1 class="text-xl font-bold text-gray-100">"Dashboard"</h1>
                {move || store.active_period.get().map(|p| {
                    let org_name = store.active_org.get().map(|o| o.name).unwrap_or_default();
                    view! {
                        <p class="text-sm text-gray-500">
                            {org_name}" · "{p.year}" · "{p.gwp_ar_version}" GWP values"
                        </p>
                    }
                })}
            </div>

            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-gray-500">"Calculating…"</p> }.into_any()
                } else if !error.get().is_empty() {
                    view! {
                        <div class="rounded-xl border border-red-800 bg-red-950/20 p-4 text-sm text-red-400">
                            {error.get()}
                        </div>
                    }.into_any()
                } else if let Some(inv) = inventory.get() {
                    view! {
                        <div>
                            // Scope summary cards
                            <div class="mb-6 grid gap-4 sm:grid-cols-3">
                                <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                                    <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">"Scope 1"</p>
                                    <p class="text-2xl font-bold text-gray-100">{fmt(Some(inv.scope1.gross_tco2e))}</p>
                                    <p class="text-xs text-gray-500">"tCO₂e · direct emissions"</p>
                                    {(inv.scope1.biogenic_co2_tco2e > 0.0).then(|| view! {
                                        <p class="mt-1 text-xs text-gray-600">
                                            "+ "{fmt(Some(inv.scope1.biogenic_co2_tco2e))}" tCO₂ biogenic (separate)"
                                        </p>
                                    })}
                                </div>

                                <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                                    <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">"Scope 2"</p>
                                    <div class="space-y-1">
                                        <div class="flex items-baseline gap-1.5">
                                            <p class="text-2xl font-bold text-gray-100">
                                                {fmt(Some(inv.scope2.location_based_tco2e))}
                                            </p>
                                            <span class="text-xs text-gray-600">"location"</span>
                                        </div>
                                        <div class="flex items-baseline gap-1.5">
                                            <p class="text-lg font-semibold text-gray-400">
                                                {fmt(Some(inv.scope2.market_based_tco2e))}
                                            </p>
                                            <span class="text-xs text-gray-600">"market"</span>
                                        </div>
                                    </div>
                                    <p class="mt-1 text-xs text-gray-500">"tCO₂e · energy indirect"</p>
                                </div>

                                <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                                    <p class="mb-1 text-xs font-semibold uppercase tracking-wider text-gray-500">"Scope 3"</p>
                                    <p class="text-2xl font-bold text-gray-100">{fmt(Some(inv.scope3.gross_tco2e))}</p>
                                    <p class="text-xs text-gray-500">"tCO₂e · other indirect"</p>
                                    <p class="mt-1 text-xs text-gray-600">
                                        {inv.scope3.excluded_categories.len()}
                                        {if inv.scope3.excluded_categories.len() == 1 { " category excluded" } else { " categories excluded" }}
                                    </p>
                                </div>
                            </div>

                            // Total
                            <div class="mb-6 rounded-xl border border-green-900/50 bg-green-950/20 p-5">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-xs font-semibold uppercase tracking-wider text-green-600">"Total (Scope 1+2+3)"</p>
                                        <p class="text-3xl font-black text-green-400">
                                            {fmt(Some(inv.total_tco2e))}
                                            " "
                                            <span class="text-base font-normal text-green-700">"tCO₂e"</span>
                                        </p>
                                    </div>
                                    <div class="text-right">
                                        <p class="text-xs text-gray-600">"Scope 1+2 only"</p>
                                        <p class="text-xl font-bold text-gray-400">
                                            {fmt(Some(inv.scope1_scope2_tco2e))}" tCO₂e"
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // Scope 3 category breakdown
                            {inv.scope3.categories.iter().any(|c| c.total_tco2e > 0.0).then(|| {
                                let cats: Vec<_> = inv.scope3.categories.iter()
                                    .filter(|c| c.total_tco2e > 0.0)
                                    .cloned()
                                    .collect();
                                view! {
                                    <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                                        <h2 class="mb-4 text-xs font-semibold uppercase tracking-wider text-gray-500">
                                            "Scope 3 Categories"
                                        </h2>
                                        <div class="space-y-1.5">
                                            {cats.into_iter().map(|cat| view! {
                                                <div class="flex items-center justify-between py-1">
                                                    <div class="flex items-center gap-2">
                                                        <span class="w-5 text-right text-xs text-gray-600">{cat.category}</span>
                                                        <span class="text-sm text-gray-300">{cat.category_name}</span>
                                                        <span class="text-xs text-gray-600">{cat.direction}</span>
                                                    </div>
                                                    <span class="text-sm font-semibold text-gray-200">
                                                        {format!("{:.3}", cat.total_tco2e)}" tCO₂e"
                                                    </span>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
                            <p class="text-sm text-gray-500">"No data yet. Add emission sources to get started."</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
