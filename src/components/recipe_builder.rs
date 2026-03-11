use dioxus::prelude::*;

use crate::{
    chemistry::compounds::Ion,
    state::{AppState, RecipeMode, RecipeSolution},
};

// ---------------------------------------------------------------------------
// Tab 2 — Recipe Builder
// ---------------------------------------------------------------------------

#[component]
pub fn RecipeBuilder() -> Element {
    let state = use_context::<Signal<AppState>>();
    let has_concentrates = !state.read().concentrates.is_empty();

    rsx! {
        div { class: "space-y-6",
            // Header
            div {
                h2 { class: "text-xl font-semibold", "Recipe Builder" }
                p { class: "text-sm text-gray-500 mt-0.5",
                    "Calculate how much of each concentrate to add to 1 L of DI water."
                }
            }

            if !has_concentrates {
                div { class: "text-center py-16 text-gray-600",
                    p { class: "text-4xl mb-3", "🧪" }
                    p { "Define your concentrates first in the Concentrates tab." }
                }
            } else {
                // Mode toggle
                ModeToggle {}

                match state.read().recipe_mode {
                    RecipeMode::TargetKhGh => rsx! { ModeTargetKhGh {} },
                    RecipeMode::ManualIons => rsx! { ModeManualIons {} },
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mode toggle
// ---------------------------------------------------------------------------

#[component]
fn ModeToggle() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mode = state.read().recipe_mode;

    let btn_base = "px-4 py-2 text-sm font-medium rounded-lg transition-colors";
    let class_kh_gh = if mode == RecipeMode::TargetKhGh {
        format!("{} bg-blue-600 text-white", btn_base)
    } else {
        format!("{} bg-gray-800 text-gray-400 hover:text-gray-200", btn_base)
    };
    let class_manual = if mode == RecipeMode::ManualIons {
        format!("{} bg-blue-600 text-white", btn_base)
    } else {
        format!("{} bg-gray-800 text-gray-400 hover:text-gray-200", btn_base)
    };

    rsx! {
        div { class: "flex gap-2 p-1 bg-gray-900 rounded-xl w-fit",
            button {
                class: "{class_kh_gh}",
                onclick: move |_| state.write().recipe_mode = RecipeMode::TargetKhGh,
                "Target KH + GH"
            }
            button {
                class: "{class_manual}",
                onclick: move |_| state.write().recipe_mode = RecipeMode::ManualIons,
                "Manual Ions"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mode A — Target KH + GH
// ---------------------------------------------------------------------------

#[component]
fn ModeTargetKhGh() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let solutions = state.read().solve_kh_gh();
    let concentrates = state.read().concentrates.clone();
    let filter = state.read().compound_filter.clone();

    rsx! {
        div { class: "space-y-6",
            // Inputs
            div { class: "bg-gray-900 border border-gray-800 rounded-xl p-5 space-y-4",
                h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide",
                    "Targets (ppm as CaCO₃)"
                }
                div { class: "grid grid-cols-2 gap-4",
                    div { class: "space-y-1",
                        label { class: "text-xs text-gray-500 uppercase tracking-wide flex items-center gap-1.5",
                            span { class: "w-2 h-2 rounded-full bg-purple-500 inline-block" }
                            "KH (Alkalinity)"
                        }
                        input {
                            r#type: "number",
                            min: "0",
                            step: "0.5",
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-purple-500",
                            value: "{state.read().target_kh}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f64>() {
                                    state.write().target_kh = v.max(0.0);
                                }
                            },
                        }
                    }
                    div { class: "space-y-1",
                        label { class: "text-xs text-gray-500 uppercase tracking-wide flex items-center gap-1.5",
                            span { class: "w-2 h-2 rounded-full bg-emerald-500 inline-block" }
                            "GH (Hardness)"
                        }
                        input {
                            r#type: "number",
                            min: "0",
                            step: "0.5",
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-emerald-500",
                            value: "{state.read().target_gh}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f64>() {
                                    state.write().target_gh = v.max(0.0);
                                }
                            },
                        }
                    }
                }

                // Concentrate filter
                if !concentrates.is_empty() {
                    div { class: "space-y-2",
                        p { class: "text-xs text-gray-500 uppercase tracking-wide",
                            "Use concentrates (all if none selected)"
                        }
                        div { class: "flex flex-wrap gap-2",
                            for conc in &concentrates {
                                {
                                    let id = conc.id;
                                    let checked = filter.contains(&id);
                                    let name = conc.name.clone();
                                    rsx! {
                                        label {
                                            class: "flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full cursor-pointer transition-colors",
                                            class: if checked { "bg-blue-800/60 text-blue-300" } else { "bg-gray-800 text-gray-400 hover:text-gray-200" },
                                            input {
                                                r#type: "checkbox",
                                                class: "hidden",
                                                checked,
                                                onchange: move |_| {
                                                    let mut s = state.write();
                                                    if s.compound_filter.contains(&id) {
                                                        s.compound_filter.remove(&id);
                                                    } else {
                                                        s.compound_filter.insert(id);
                                                    }
                                                },
                                            }
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Solutions
            div { class: "space-y-3",
                if solutions.is_empty() {
                    div { class: "text-center py-8 text-gray-600 bg-gray-900 rounded-xl border border-gray-800",
                        p { "No solution found with the selected concentrates." }
                        p { class: "text-xs mt-1",
                            "Add a KH concentrate and/or a GH concentrate, or clear the filter."
                        }
                    }
                } else {
                    h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide",
                        "Solutions ({solutions.len()})"
                    }
                    for (i, sol) in solutions.iter().enumerate() {
                        SolutionCard {
                            key: "{i}",
                            index: i + 1,
                            solution: sol.clone(),
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mode B — Manual Ions
// ---------------------------------------------------------------------------

#[component]
fn ModeManualIons() -> Element {
    let state = use_context::<Signal<AppState>>();
    let solution = state.read().solve_manual();

    // Which ions can be targeted (only those with an available concentrate)
    let available_ions: Vec<Ion> = {
        let s = state.read();
        let mut ions = vec![];
        for ion in [Ion::Ca, Ion::Mg, Ion::HCO3, Ion::Na, Ion::K] {
            if s.concentrates.iter().any(|c| c.primary_ion() == ion) {
                ions.push(ion);
            }
        }
        ions
    };

    rsx! {
        div { class: "space-y-6",
            // Ion inputs
            div { class: "bg-gray-900 border border-gray-800 rounded-xl p-5 space-y-4",
                h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide",
                    "Ion targets (ppm as CaCO₃ per 1 L)"
                }

                if available_ions.is_empty() {
                    p { class: "text-sm text-gray-600",
                        "No concentrates with targetable primary ions defined yet."
                    }
                } else {
                    div { class: "space-y-3",
                        for ion in &available_ions {
                            IonRow { ion: *ion }
                        }
                    }
                }
            }

            // Result
            if !solution.ions.is_empty() {
                SolutionCard {
                    index: 0,
                    solution: solution,
                }
            }
        }
    }
}

#[component]
fn IonRow(ion: Ion) -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let value = {
        let s = state.read();
        match ion {
            Ion::Ca => s.target_ca,
            Ion::Mg => s.target_mg,
            Ion::HCO3 => s.target_hco3,
            Ion::Na => s.target_na,
            Ion::K => s.target_k,
            _ => 0.0,
        }
    };

    // Which concentrates serve this ion
    let conc_names: Vec<String> = state
        .read()
        .concentrates
        .iter()
        .filter(|c| c.primary_ion() == ion)
        .map(|c| c.name.clone())
        .collect();

    rsx! {
        div { class: "flex items-center gap-3",
            div { class: "w-16 text-sm font-mono text-gray-300", "{ion.label()}" }
            input {
                r#type: "number",
                min: "0",
                step: "0.5",
                class: "w-28 bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:border-blue-500",
                value: "{value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        let v = v.max(0.0);
                        let mut s = state.write();
                        match ion {
                            Ion::Ca => s.target_ca = v,
                            Ion::Mg => s.target_mg = v,
                            Ion::HCO3 => s.target_hco3 = v,
                            Ion::Na => s.target_na = v,
                            Ion::K => s.target_k = v,
                            _ => {}
                        }
                    }
                },
            }
            span { class: "text-xs text-gray-600",
                "→ {conc_names.join(\", \")}"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Solution Card (shared by both modes)
// ---------------------------------------------------------------------------

#[component]
fn SolutionCard(index: usize, solution: RecipeSolution) -> Element {
    let state = use_context::<Signal<AppState>>(); // read-only

    rsx! {
        div { class: "bg-gray-900 border border-gray-800 rounded-xl p-4 space-y-4",
            // Header with KH/GH summary
            div { class: "flex items-center justify-between",
                if index > 0 {
                    span { class: "text-xs text-gray-600", "Option {index}" }
                } else {
                    span { class: "text-xs text-gray-600", "Result" }
                }
                div { class: "flex gap-3",
                    div { class: "flex items-center gap-1.5",
                        span { class: "w-2 h-2 rounded-full bg-purple-500 inline-block" }
                        span { class: "text-sm font-mono font-semibold text-gray-200",
                            "KH {solution.kh:.2} ppm"
                        }
                    }
                    div { class: "flex items-center gap-1.5",
                        span { class: "w-2 h-2 rounded-full bg-emerald-500 inline-block" }
                        span { class: "text-sm font-mono font-semibold text-gray-200",
                            "GH {solution.gh:.2} ppm"
                        }
                    }
                }
            }

            // Concentrate amounts
            if !solution.usages.is_empty() {
                div { class: "space-y-1",
                    p { class: "text-xs text-gray-600 uppercase tracking-wide", "Add to 1 L DI water" }
                    for usage in &solution.usages {
                        {
                            let conc_name = state
                                .read()
                                .concentrate_by_id(usage.concentrate_id)
                                .map(|c| c.name.clone())
                                .unwrap_or_else(|| "?".to_string());
                            rsx! {
                                div { class: "flex justify-between text-sm",
                                    span { class: "text-gray-400", "{conc_name}" }
                                    span { class: "font-mono font-semibold text-blue-400",
                                        {format!("{:.3} mL", usage.ml_per_liter)}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Full ion breakdown
            div { class: "border-t border-gray-800 pt-3",
                p { class: "text-xs text-gray-600 uppercase tracking-wide mb-2",
                    "Final ion profile (per 1 L)"
                }
                div { class: "grid grid-cols-2 gap-x-6 gap-y-1",
                    for (ion, ppm, mg) in &solution.ions {
                        div { class: "flex justify-between text-xs",
                            span {
                                class: {
                                    if ion.contributes_to_kh() {
                                        "text-purple-400"
                                    } else if ion.contributes_to_gh() {
                                        "text-emerald-400"
                                    } else {
                                        "text-gray-500"
                                    }
                                },
                                "{ion.label()}"
                            }
                            span { class: "font-mono text-gray-300",
                                {format!("{:.2} ppm  ({:.2} mg/L)", ppm, mg)}
                            }
                        }
                    }
                }
            }
        }
    }
}
