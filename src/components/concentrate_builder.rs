use dioxus::prelude::*;

use crate::{
    chemistry::compounds::Compound,
    state::{AppState, Concentrate},
    storage,
};

// ---------------------------------------------------------------------------
// Tab 1 — Concentrate Builder
// ---------------------------------------------------------------------------

#[component]
pub fn ConcentrateBuilder() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut show_form = use_signal(|| false);

    rsx! {
        div { class: "space-y-6",
            // Header row
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-xl font-semibold", "Concentrates" }
                    p { class: "text-sm text-gray-500 mt-0.5",
                        "Define stock solutions. Each concentrate maps to one compound."
                    }
                }
                button {
                    class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors",
                    onclick: move |_| show_form.set(!show_form()),
                    if show_form() { "Cancel" } else { "+ Add Concentrate" }
                }
            }

            // Add form
            if show_form() {
                AddConcentrateForm {
                    on_save: move |c: Concentrate| {
                        state.write().add_concentrate(c);
                        let s = state.read();
                        storage::save_concentrates(&s.concentrates, s.next_id);
                        show_form.set(false);
                    }
                }
            }

            // Existing concentrates
            {
                let concs = state.read().concentrates.clone();
                if concs.is_empty() && !show_form() {
                    rsx! {
                        div { class: "text-center py-16 text-gray-600",
                            p { class: "text-4xl mb-3", "⚗️" }
                            p { "No concentrates yet. Add one to get started." }
                        }
                    }
                } else {
                    rsx! {
                        // Group by what they affect
                        ConcentrateGroup {
                            label: "GH Concentrates",
                            description: "Raise general hardness (Ca²⁺ / Mg²⁺)",
                            concentrates: concs.iter().filter(|c| c.compound.affects_gh()).cloned().collect::<Vec<_>>(),
                            on_remove: move |id| {
                                state.write().remove_concentrate(id);
                                let s = state.read();
                                storage::save_concentrates(&s.concentrates, s.next_id);
                            },
                        }
                        ConcentrateGroup {
                            label: "KH Concentrates",
                            description: "Raise carbonate hardness / alkalinity (HCO₃⁻)",
                            concentrates: concs.iter().filter(|c| c.compound.affects_kh()).cloned().collect::<Vec<_>>(),
                            on_remove: move |id| {
                                state.write().remove_concentrate(id);
                                let s = state.read();
                                storage::save_concentrates(&s.concentrates, s.next_id);
                            },
                        }
                        ConcentrateGroup {
                            label: "Other",
                            description: "Salinity / flavour (no effect on KH or GH)",
                            concentrates: concs.iter().filter(|c| !c.compound.affects_gh() && !c.compound.affects_kh()).cloned().collect::<Vec<_>>(),
                            on_remove: move |id| {
                                state.write().remove_concentrate(id);
                                let s = state.read();
                                storage::save_concentrates(&s.concentrates, s.next_id);
                            },
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Group of concentrate cards
// ---------------------------------------------------------------------------

#[component]
fn ConcentrateGroup(
    label: &'static str,
    description: &'static str,
    concentrates: Vec<Concentrate>,
    on_remove: EventHandler<u32>,
) -> Element {
    if concentrates.is_empty() {
        return rsx! { Fragment {} };
    }

    rsx! {
        div { class: "space-y-3",
            div {
                h3 { class: "text-xs font-semibold uppercase tracking-widest text-gray-500", "{label}" }
                p { class: "text-xs text-gray-600", "{description}" }
            }
            div { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-3",
                for conc in concentrates {
                    ConcentrateCard {
                        key: "{conc.id}",
                        concentrate: conc.clone(),
                        on_remove: move |_| on_remove.call(conc.id),
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Concentrate Card
// ---------------------------------------------------------------------------

#[component]
fn ConcentrateCard(concentrate: Concentrate, on_remove: EventHandler<()>) -> Element {
    let grams = concentrate.grams_needed();
    let profile = concentrate.ion_profile_per_ml();
    let formula = concentrate.formula();

    rsx! {
        div { class: "bg-gray-900 border border-gray-800 rounded-xl p-4 space-y-3",
            // Card header
            div { class: "flex items-start justify-between gap-2",
                div {
                    p { class: "font-medium text-gray-100", "{concentrate.name}" }
                    p { class: "text-xs text-gray-500 font-mono", "{formula}" }
                }
                button {
                    class: "text-gray-600 hover:text-red-400 transition-colors text-lg leading-none mt-0.5",
                    onclick: move |_| on_remove.call(()),
                    "×"
                }
            }

            // Potency + grams
            div { class: "bg-gray-950 rounded-lg p-3 space-y-1",
                div { class: "flex justify-between text-sm",
                    span { class: "text-gray-500", "Potency" }
                    span {
                        class: "font-mono text-gray-200",
                        title: "{concentrate.compound.primary_ion().label()} as CaCO₃",
                        {format!("{:.2} ppm/mL", concentrate.potency_ppm_per_ml)}
                    }
                }
                div { class: "flex justify-between text-sm",
                    span { class: "text-gray-500", "Stock volume" }
                    span { class: "font-mono text-gray-200",
                        {format!("{:.0} mL", concentrate.stock_volume_ml)}
                    }
                }
                div { class: "border-t border-gray-800 pt-1 mt-1 flex justify-between text-sm",
                    span { class: "text-gray-400 font-medium", "Dissolve" }
                    span { class: "font-mono text-blue-400 font-semibold",
                        {format!("{:.3} g", grams)}
                    }
                }
            }

            // Full ion profile
            div { class: "space-y-1",
                p { class: "text-xs text-gray-600 uppercase tracking-wide", "Ion profile (per mL → 1 L)" }
                for (ion, ppm, mg) in &profile {
                    div { class: "flex justify-between text-xs",
                        span { class: "text-gray-500", "{ion.label()}" }
                        span { class: "font-mono text-gray-300",
                            {format!("{:.2} ppm  ({:.2} mg/L)", ppm, mg)}
                        }
                    }
                }
            }

            // Affects badge
            div {
                span {
                    class: {
                        let base = "inline-block text-xs px-2 py-0.5 rounded-full font-medium";
                        if concentrate.compound.affects_gh() {
                            format!("{} bg-emerald-900/50 text-emerald-400", base)
                        } else if concentrate.compound.affects_kh() {
                            format!("{} bg-purple-900/50 text-purple-400", base)
                        } else {
                            format!("{} bg-gray-800 text-gray-400", base)
                        }
                    },
                    "Affects: {concentrate.compound.affects_label()}"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Add Concentrate Form
// ---------------------------------------------------------------------------

#[component]
fn AddConcentrateForm(on_save: EventHandler<Concentrate>) -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut compound = use_signal(|| Compound::CaCl2);
    let mut use_anhydrous = use_signal(|| false);
    let mut potency = use_signal(|| 10.0_f64);
    let mut stock_volume = use_signal(|| 500.0_f64);

    // Preview
    let preview_grams = {
        let c = Concentrate {
            id: 0,
            name: name.read().clone(),
            compound: compound(),
            use_anhydrous: use_anhydrous(),
            potency_ppm_per_ml: potency(),
            stock_volume_ml: stock_volume(),
        };
        c.grams_needed()
    };

    let preview_profile = {
        crate::chemistry::conversions::ion_profile_per_ml(compound(), potency())
    };

    let can_save = !name.read().trim().is_empty() && potency() > 0.0 && stock_volume() > 0.0;

    rsx! {
        div { class: "bg-gray-900 border border-blue-800/50 rounded-xl p-5 space-y-4",
            h3 { class: "font-semibold text-gray-200", "New Concentrate" }

            // Compound selector
            div { class: "space-y-1",
                label { class: "text-xs text-gray-500 uppercase tracking-wide", "Compound" }
                select {
                    class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500",
                    oninput: move |e| {
                        let val = e.value();
                        compound.set(match val.as_str() {
                            "MgSO4" => Compound::MgSO4,
                            "MgCl2" => Compound::MgCl2,
                            "NaHCO3" => Compound::NaHCO3,
                            "KHCO3" => Compound::KHCO3,
                            "NaCl" => Compound::NaCl,
                            _ => Compound::CaCl2,
                        });
                        // Reset anhydrous when compound changes
                        use_anhydrous.set(false);
                    },
                    option { value: "CaCl2", "Calcium Chloride (CaCl₂) — GH (Ca²⁺)" }
                    option { value: "MgSO4", "Magnesium Sulfate / Epsom (MgSO₄) — GH (Mg²⁺)" }
                    option { value: "MgCl2", "Magnesium Chloride (MgCl₂) — GH (Mg²⁺)" }
                    option { value: "NaHCO3", "Sodium Bicarbonate (NaHCO₃) — KH" }
                    option { value: "KHCO3", "Potassium Bicarbonate (KHCO₃) — KH" }
                    option { value: "NaCl", "Sodium Chloride / Salt (NaCl) — Salinity" }
                }
            }

            // Hydrated / Anhydrous toggle (only when relevant)
            if compound().has_hydrated_form() {
                div { class: "flex items-center gap-3",
                    label { class: "text-xs text-gray-500 uppercase tracking-wide", "Form" }
                    div { class: "flex gap-4",
                        label { class: "flex items-center gap-1.5 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "radio",
                                name: "form",
                                checked: !use_anhydrous(),
                                onchange: move |_| use_anhydrous.set(false),
                                class: "accent-blue-500",
                            }
                            "Hydrated"
                            span { class: "text-gray-600 font-mono text-xs",
                                "({compound().formula_anhydrous()}{compound().hydration_label().unwrap_or(\"\")})"
                            }
                        }
                        label { class: "flex items-center gap-1.5 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "radio",
                                name: "form",
                                checked: use_anhydrous(),
                                onchange: move |_| use_anhydrous.set(true),
                                class: "accent-blue-500",
                            }
                            "Anhydrous"
                            span { class: "text-gray-600 font-mono text-xs",
                                "({compound().formula_anhydrous()})"
                            }
                        }
                    }
                }
            }

            // Name
            div { class: "space-y-1",
                label { class: "text-xs text-gray-500 uppercase tracking-wide", "Name" }
                input {
                    r#type: "text",
                    class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500",
                    placeholder: "e.g. Ca stock",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
            }

            // Two-column: potency + stock volume
            div { class: "grid grid-cols-2 gap-3",
                div { class: "space-y-1",
                    label { class: "text-xs text-gray-500 uppercase tracking-wide",
                        "Potency (ppm/mL)"
                    }
                    input {
                        r#type: "number",
                        min: "0.01",
                        step: "0.1",
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500",
                        value: "{potency}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                potency.set(v);
                            }
                        },
                    }
                    p { class: "text-xs text-gray-600",
                        {format!("{} ppm-as-CaCO₃ of {} per mL → 1 L",
                            potency(),
                            compound().primary_ion().label())}
                    }
                }

                div { class: "space-y-1",
                    label { class: "text-xs text-gray-500 uppercase tracking-wide",
                        "Stock Volume (mL)"
                    }
                    input {
                        r#type: "number",
                        min: "1",
                        step: "1",
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-blue-500",
                        value: "{stock_volume}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                stock_volume.set(v);
                            }
                        },
                    }
                }
            }

            // Live preview
            div { class: "bg-gray-950 rounded-lg p-3 space-y-2",
                p { class: "text-xs text-gray-600 uppercase tracking-wide", "Preview" }
                div { class: "flex justify-between text-sm",
                    span { class: "text-gray-400", "Dissolve" }
                    span { class: "font-mono text-blue-400 font-semibold",
                        {format!("{:.3} g in {:.0} mL", preview_grams, stock_volume())}
                    }
                }
                div { class: "border-t border-gray-800 pt-2 space-y-1",
                    p { class: "text-xs text-gray-600", "Ion profile per mL → 1 L:" }
                    for (ion, ppm, mg) in &preview_profile {
                        div { class: "flex justify-between text-xs",
                            span { class: "text-gray-500", "{ion.label()}" }
                            span { class: "font-mono text-gray-300",
                                {format!("{:.2} ppm  ({:.2} mg/L)", ppm, mg)}
                            }
                        }
                    }
                }
            }

            // Save button
            button {
                class: "w-full py-2 rounded-lg text-sm font-medium transition-colors",
                class: if can_save {
                    "bg-blue-600 hover:bg-blue-500 text-white"
                } else {
                    "bg-gray-800 text-gray-600 cursor-not-allowed"
                },
                disabled: !can_save,
                onclick: move |_| {
                    if can_save {
                        on_save.call(Concentrate {
                            id: 0, // assigned by AppState
                            name: name.read().trim().to_string(),
                            compound: compound(),
                            use_anhydrous: use_anhydrous(),
                            potency_ppm_per_ml: potency(),
                            stock_volume_ml: stock_volume(),
                        });
                    }
                },
                "Save Concentrate"
            }
        }
    }
}
