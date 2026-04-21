use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

use crate::store::AppStore;
use crate::tauri;
use crate::types::{Cop, CopQuestion};

const SECTION_LABELS: &[(&str, &str)] = &[
    ("E", "Environment (Principles 7–9)"),
    ("G", "Governance & Anti-Corruption (Principle 10)"),
    ("L", "Labour Standards (Principles 3–6)"),
    ("H", "Human Rights (Principles 1–2)"),
];

fn level_color(level: &str) -> &'static str {
    match level {
        "lead"     => "rounded-full border px-3 py-1 text-xs font-semibold uppercase text-purple-400 border-purple-800 bg-purple-950/30",
        "advanced" => "rounded-full border px-3 py-1 text-xs font-semibold uppercase text-blue-400 border-blue-800 bg-blue-950/30",
        "active"   => "rounded-full border px-3 py-1 text-xs font-semibold uppercase text-green-400 border-green-800 bg-green-950/30",
        _          => "rounded-full border px-3 py-1 text-xs font-semibold uppercase text-yellow-400 border-yellow-800 bg-yellow-950/30",
    }
}

#[component]
pub fn Ungc() -> impl IntoView {
    let store = use_context::<AppStore>().expect("AppStore not provided");
    let cop = RwSignal::new(None::<Cop>);
    let questions = RwSignal::new(Vec::<CopQuestion>::new());
    let compliance_level = RwSignal::new(None::<String>);
    let ceo_name = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let saving = RwSignal::new(None::<String>);
    let error = RwSignal::new(String::new());
    let auto_populating = RwSignal::new(false);

    // Group questions by section prefix
    let by_section = Memo::new(move |_| {
        let mut result: Vec<(String, Vec<CopQuestion>)> = Vec::new();
        let mut map: HashMap<String, Vec<CopQuestion>> = HashMap::new();
        for q in questions.get() {
            let section = q.question_id.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "O".to_string());
            map.entry(section).or_default().push(q);
        }
        // maintain E, G, L, H order
        for (key, _) in SECTION_LABELS {
            if let Some(qs) = map.remove(*key) {
                result.push((key.to_string(), qs));
            }
        }
        // Any remaining sections
        for (k, v) in map {
            result.push((k, v));
        }
        result
    });

    Effect::new(move |_| {
        let period = store.active_period.get();
        let org = store.active_org.get();
        if let (Some(period), Some(org)) = (period, org) {
            loading.set(true);
            error.set(String::new());
            spawn_local(async move {
                match tauri::init_cop(org.id, period.year).await {
                    Ok(c) => {
                        let cop_id = c.id;
                        cop.set(Some(c));
                        match tauri::get_cop_questions(cop_id).await {
                            Ok(qs) => questions.set(qs),
                            Err(e) => error.set(e),
                        }
                        match tauri::compute_compliance_level(cop_id).await {
                            Ok(l) => compliance_level.set(Some(l)),
                            Err(_) => {}
                        }
                    }
                    Err(e) => error.set(e),
                }
                loading.set(false);
            });
        }
    });

    let populate = move |_| {
        let c = cop.get();
        let period = store.active_period.get();
        if c.is_none() || period.is_none() { return; }
        let c = c.unwrap();
        let period = period.unwrap();
        auto_populating.set(true);
        error.set(String::new());
        spawn_local(async move {
            let _ = tauri::auto_populate_cop(c.id, period.id).await;
            if let Ok(qs) = tauri::get_cop_questions(c.id).await { questions.set(qs); }
            if let Ok(l) = tauri::compute_compliance_level(c.id).await { compliance_level.set(Some(l)); }
            auto_populating.set(false);
        });
    };

    let save_response = move |q: CopQuestion, value: String| {
        let c = cop.get();
        if c.is_none() { return; }
        let c = c.unwrap();
        let qid = q.question_id.clone();
        saving.set(Some(qid.clone()));
        spawn_local(async move {
            let _ = tauri::save_cop_response(c.id, &qid, &value).await;
            questions.update(|qs| {
                for x in qs.iter_mut() {
                    if x.question_id == qid {
                        x.response = Some(value.clone());
                        break;
                    }
                }
            });
            if let Ok(l) = tauri::compute_compliance_level(c.id).await { compliance_level.set(Some(l)); }
            saving.set(None);
        });
    };

    let sign_statement = move |_| {
        let c = cop.get();
        let name = ceo_name.get();
        if c.is_none() || name.trim().is_empty() {
            error.set("CEO name is required".into());
            return;
        }
        let c = c.unwrap();
        error.set(String::new());
        spawn_local(async move {
            match tauri::sign_ceo_statement(c.id, &name.trim()).await {
                Ok(_) => {
                    cop.update(|c| { if let Some(c) = c { c.ceo_statement_signed = true; } });
                    if let Ok(l) = tauri::compute_compliance_level(c.id).await { compliance_level.set(Some(l)); }
                }
                Err(e) => error.set(e),
            }
        });
    };

    let answered_count = move || {
        questions.with(|qs| qs.iter().filter(|q| q.response.as_deref().map(|r| !r.trim().is_empty()).unwrap_or(false)).count())
    };

    view! {
        <div class="p-8">
            <div class="mb-6">
                <h1 class="text-xl font-bold text-gray-100">"UNGC Communication on Progress"</h1>
                <p class="text-xs text-gray-500">
                    "Annual submission · Principles 7–9 (Environment) · 2025 questionnaire format"
                </p>
            </div>

            {move || loading.get().then(|| view! { <p class="text-sm text-gray-500">"Loading COP…"</p> })}
            {move || (!error.get().is_empty()).then(|| view! {
                <div class="mb-4 rounded-xl border border-red-800 bg-red-950/20 p-4 text-sm text-red-400">{error.get()}</div>
            })}

            {move || cop.get().map(|c| {
                let _cop_id = c.id;
                view! {
                    <div>
                        // Status bar
                        <div class="mb-6 flex items-center gap-4">
                            {move || compliance_level.get().map(|l| view! {
                                <span class=level_color(&l)>{l.clone()}</span>
                            })}
                            <span class="text-xs text-gray-500">
                                {answered_count()}" / "{questions.with(|qs| qs.len())}" questions answered"
                            </span>
                            {c.ceo_statement_signed.then(|| view! {
                                <span class="text-xs text-green-500">"✓ CEO Statement signed"</span>
                            })}
                            <div class="ml-auto flex gap-2">
                                <button
                                    on:click=populate
                                    disabled=move || auto_populating.get()
                                    class="rounded-lg border border-gray-700 px-4 py-2 text-xs text-gray-400 hover:border-gray-600 disabled:opacity-50"
                                >
                                    {move || if auto_populating.get() { "Populating…" } else { "Auto-populate from GRI 305" }}
                                </button>
                            </div>
                        </div>

                        // CEO Statement
                        {(!c.ceo_statement_signed).then(|| view! {
                            <div class="mb-6 rounded-xl border border-yellow-700/50 bg-yellow-950/20 p-5">
                                <h2 class="mb-3 text-sm font-semibold text-yellow-400">"CEO Statement of Continued Support"</h2>
                                <p class="mb-4 text-xs text-yellow-300/70">
                                    "Required for all UNGC COP submissions. The CEO (or equivalent) must affirm continued commitment to "
                                    "the Ten Principles. This cannot be submitted without a signed statement."
                                </p>
                                <div class="flex gap-3">
                                    <input
                                        type="text"
                                        placeholder="CEO full name"
                                        prop:value=move || ceo_name.get()
                                        on:input=move |ev| ceo_name.set(event_target_value(&ev))
                                        class="flex-1 rounded-lg border border-yellow-700/50 bg-gray-800 px-3 py-2 text-sm text-gray-100 placeholder:text-gray-600 focus:border-yellow-600 focus:outline-none"
                                    />
                                    <button
                                        on:click=sign_statement
                                        class="rounded-lg bg-yellow-600 px-4 py-2 text-sm font-semibold text-white hover:bg-yellow-700"
                                    >
                                        "Sign Statement"
                                    </button>
                                </div>
                                <p class="mt-3 text-[10px] text-gray-600">
                                    "By signing, "
                                    {move || { let n = ceo_name.get(); if n.is_empty() { "[CEO name]".to_string() } else { n } }}
                                    " confirms continued commitment to the UNGC Ten Principles."
                                </p>
                            </div>
                        })}

                        {c.ceo_statement_signed.then(|| view! {
                            <div class="mb-6 rounded-xl border border-green-800/40 bg-green-950/15 p-4">
                                <p class="text-sm text-green-400">"✓ CEO Statement signed — ready for submission"</p>
                            </div>
                        })}

                        // Question sections
                        {move || by_section.get().into_iter().map(|(section, qs)| {
                            let label = SECTION_LABELS.iter()
                                .find(|(k, _)| *k == section)
                                .map(|(_, l)| *l)
                                .unwrap_or(&section);
                            let label = label.to_string();
                            view! {
                                <div class="mb-6">
                                    <h2 class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">
                                        {label}
                                    </h2>
                                    <div class="space-y-2">
                                        {qs.into_iter().map(|q| {
                                            let qid = q.question_id.clone();
                                            let _response_init = q.response.clone().unwrap_or_default();
                                            let q_clone = q.clone();
                                            view! {
                                                <div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
                                                    <div class="mb-2 flex items-start justify-between gap-4">
                                                        <div class="flex items-start gap-2">
                                                            <span class="mt-0.5 font-mono text-xs text-gray-600">{q.question_id.clone()}</span>
                                                            <p class="text-sm text-gray-300">{q.question_text.clone()}</p>
                                                        </div>
                                                        {q.auto_populated.then(|| view! {
                                                            <span class="shrink-0 rounded-full border border-green-800/50 bg-green-950/30 px-2 py-0.5 text-[10px] text-green-500">
                                                                "auto"
                                                            </span>
                                                        })}
                                                    </div>
                                                    <textarea
                                                        rows="3"
                                                        placeholder="Enter your response…"
                                                        prop:value=move || {
                                                            questions.with(|qs| {
                                                                qs.iter()
                                                                    .find(|x| x.question_id == qid)
                                                                    .and_then(|x| x.response.clone())
                                                                    .unwrap_or_default()
                                                            })
                                                        }
                                                        on:blur=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            let q2 = q_clone.clone();
                                                            save_response(q2, val);
                                                        }
                                                        class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100 placeholder:text-gray-600 focus:border-green-600 focus:outline-none resize-none"
                                                    ></textarea>
                                                    {move || {
                                                        let sid = saving.get();
                                                        let qid2 = q.question_id.clone();
                                                        let resp = questions.with(|qs| {
                                                            qs.iter().find(|x| x.question_id == qid2).and_then(|x| x.response.clone())
                                                        });
                                                        if sid.as_deref() == Some(&qid2) {
                                                            view! { <p class="mt-1 text-[10px] text-gray-600">"Saving…"</p> }.into_any()
                                                        } else if resp.as_deref().map(|r| !r.is_empty()).unwrap_or(false) {
                                                            view! { <p class="mt-1 text-[10px] text-green-700">"✓ Answered"</p> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }
                                                    }}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }
                        }).collect_view()}

                        // Compliance levels explainer
                        <div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
                            <h2 class="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">"Compliance Levels"</h2>
                            <div class="space-y-2">
                                {vec![
                                    ("beginner", "CEO Statement + any responses submitted", "text-yellow-400"),
                                    ("active",   "CEO Statement + ≥50% of questions answered", "text-green-400"),
                                    ("advanced", "CEO Statement + ≥75% answered + environmental & social coverage", "text-blue-400"),
                                    ("lead",     "CEO Statement + ≥90% answered + all principle areas covered", "text-purple-400"),
                                ].into_iter().map(|(level, desc, color)| view! {
                                    <div class="flex items-start gap-3">
                                        <span class=format!("w-16 shrink-0 text-xs font-semibold uppercase {color}")>{level}</span>
                                        <span class="text-xs text-gray-500">{desc}</span>
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
