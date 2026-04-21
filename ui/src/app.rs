use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::sidebar::Sidebar;
use crate::routes::{
    dashboard::Dashboard, reports::Reports, scope1::Scope1, scope2::Scope2, scope3::Scope3,
    settings::Settings, setup::Setup, ungc::Ungc,
};
use crate::store::AppStore;
use crate::tauri;

#[component]
pub fn App() -> impl IntoView {
    let store = AppStore::new();
    provide_context(store);

    let loading = RwSignal::new(true);

    Effect::new(move |_| {
        spawn_local(async move {
            match tauri::list_orgs().await {
                Ok(orgs) if !orgs.is_empty() => {
                    let org = orgs.into_iter().next().unwrap();
                    let org_id = org.id;
                    store.active_org.set(Some(org));
                    if let Ok(periods) = tauri::list_periods(org_id).await {
                        if let Some(period) = periods.into_iter().next() {
                            store.active_period.set(Some(period));
                        }
                    }
                    store.navigate("/dashboard");
                }
                _ => {
                    store.navigate("/setup");
                }
            }
            loading.set(false);
        });
    });

    view! {
        {move || {
            if loading.get() {
                view! {
                    <div class="flex h-full items-center justify-center bg-gray-950">
                        <div class="text-center">
                            <div class="mb-3 text-2xl font-bold text-green-500">"C12"</div>
                            <div class="text-sm text-gray-500">"Loading…"</div>
                        </div>
                    </div>
                }.into_any()
            } else {
                let route = store.current_route;
                view! {
                    {move || {
                        if route.get() == "/setup" {
                            view! { <Setup /> }.into_any()
                        } else {
                            view! {
                                <div class="flex h-full overflow-hidden bg-gray-950">
                                    <Sidebar />
                                    <main class="flex-1 overflow-y-auto">
                                        {move || match route.get().as_str() {
                                            "/dashboard"      => view! { <Dashboard /> }.into_any(),
                                            "/sources/scope1" => view! { <Scope1 /> }.into_any(),
                                            "/sources/scope2" => view! { <Scope2 /> }.into_any(),
                                            "/sources/scope3" => view! { <Scope3 /> }.into_any(),
                                            "/reports"        => view! { <Reports /> }.into_any(),
                                            "/ungc"           => view! { <Ungc /> }.into_any(),
                                            "/settings"       => view! { <Settings /> }.into_any(),
                                            _                 => view! { <Dashboard /> }.into_any(),
                                        }}
                                    </main>
                                </div>
                            }.into_any()
                        }
                    }}
                }.into_any()
            }
        }}
    }
}
