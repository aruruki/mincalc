use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::chemistry::{
    compounds::{Compound, Ion},
    conversions::{ion_profile_per_ml, ml_needed, potency_to_grams},
};

// ---------------------------------------------------------------------------
// Tab
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Concentrates,
    Recipe,
}

// ---------------------------------------------------------------------------
// Concentrate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concentrate {
    pub id: u32,
    pub name: String,
    pub compound: Compound,
    pub use_anhydrous: bool,
    /// ppm-as-CaCO3 of primary ion when 1 mL of this stock is added to 1 L.
    pub potency_ppm_per_ml: f64,
    /// How many mL of stock to prepare.
    pub stock_volume_ml: f64,
}

impl Concentrate {
    /// Grams of compound to dissolve in `stock_volume_ml` mL of DI water.
    pub fn grams_needed(&self) -> f64 {
        potency_to_grams(
            self.compound,
            self.potency_ppm_per_ml,
            self.stock_volume_ml,
            self.use_anhydrous,
        )
    }

    /// Full ion profile (ppm-as-CaCO3, mg/L) when 1 mL is added to 1 L.
    pub fn ion_profile_per_ml(&self) -> Vec<(Ion, f64, f64)> {
        ion_profile_per_ml(self.compound, self.potency_ppm_per_ml)
    }

    /// ppm-as-CaCO3 of this concentrate's primary ion per mL → 1 L.
    pub fn primary_ppm_per_ml(&self) -> f64 {
        self.potency_ppm_per_ml
    }

    pub fn primary_ion(&self) -> Ion {
        self.compound.primary_ion()
    }

    /// Display formula string (hydrated or anhydrous).
    pub fn formula(&self) -> String {
        let base = self.compound.formula_anhydrous();
        if self.use_anhydrous || !self.compound.has_hydrated_form() {
            base.to_string()
        } else {
            format!("{}{}", base, self.compound.hydration_label().unwrap_or(""))
        }
    }
}

// ---------------------------------------------------------------------------
// Recipe Mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeMode {
    TargetKhGh,
    ManualIons,
}

// ---------------------------------------------------------------------------
// Recipe Solution (output of solver)
// ---------------------------------------------------------------------------

/// One line in a recipe solution: how many mL of a concentrate to add to 1 L.
#[derive(Debug, Clone, PartialEq)]
pub struct ConcentrateUsage {
    pub concentrate_id: u32,
    pub ml_per_liter: f64,
}

/// A complete solution for a recipe (set of concentrate usages + resulting
/// ion profile).
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeSolution {
    pub usages: Vec<ConcentrateUsage>,
    /// Final ion concentrations: (Ion, ppm-as-CaCO3, mg/L).
    pub ions: Vec<(Ion, f64, f64)>,
    pub kh: f64,
    pub gh: f64,
}

// ---------------------------------------------------------------------------
// App-level state (held in Dioxus signals via context)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub active_tab: Tab,
    pub concentrates: Vec<Concentrate>,
    pub next_id: u32,

    // ---- Recipe state ----
    pub recipe_mode: RecipeMode,

    // Mode A — target KH + GH
    pub target_kh: f64,
    pub target_gh: f64,
    /// Concentrate IDs the user wants to use; empty = use all.
    pub compound_filter: HashSet<u32>,

    // Mode B — manual ions
    pub target_ca: f64,
    pub target_mg: f64,
    pub target_hco3: f64,
    pub target_na: f64,
    pub target_k: f64,
    /// Concentrate IDs the user wants to use for manual ion mode; empty = use all.
    pub manual_ion_filter: HashSet<u32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_tab: Tab::Concentrates,
            concentrates: vec![],
            next_id: 1,
            recipe_mode: RecipeMode::TargetKhGh,
            target_kh: 0.0,
            target_gh: 0.0,
            compound_filter: HashSet::new(),
            target_ca: 0.0,
            target_mg: 0.0,
            target_hco3: 0.0,
            target_na: 0.0,
            target_k: 0.0,
            manual_ion_filter: HashSet::new(),
        }
    }
}

impl AppState {
    pub fn add_concentrate(&mut self, mut c: Concentrate) {
        c.id = self.next_id;
        self.next_id += 1;
        self.concentrates.push(c);
    }

    pub fn remove_concentrate(&mut self, id: u32) {
        self.concentrates.retain(|c| c.id != id);
        self.compound_filter.remove(&id);
        self.manual_ion_filter.remove(&id);
    }

    pub fn concentrate_by_id(&self, id: u32) -> Option<&Concentrate> {
        self.concentrates.iter().find(|c| c.id == id)
    }

    // -----------------------------------------------------------------------
    // Solver — Mode A: target KH + GH
    // -----------------------------------------------------------------------

    /// Returns all valid recipe solutions that satisfy `target_kh` and
    /// `target_gh` using the concentrates selected in `compound_filter`
    /// (or all concentrates if the filter is empty).
    ///
    /// Strategy:
    ///   - KH concentrates: those whose primary ion is HCO3.
    ///   - GH concentrates: those whose primary ion is Ca or Mg.
    ///   - No compound crosses the boundary, so the problems are independent.
    ///   - For KH: enumerate single-concentrate solutions, then pairs.
    ///   - For GH: same.
    ///   - Cross-product KH solutions × GH solutions = full solutions.
    pub fn solve_kh_gh(&self) -> Vec<RecipeSolution> {
        let available: Vec<&Concentrate> = self
            .concentrates
            .iter()
            .filter(|c| {
                self.compound_filter.is_empty() || self.compound_filter.contains(&c.id)
            })
            .collect();

        let kh_concs: Vec<&Concentrate> = available
            .iter()
            .filter(|c| c.primary_ion().contributes_to_kh())
            .copied()
            .collect();

        let gh_concs: Vec<&Concentrate> = available
            .iter()
            .filter(|c| c.primary_ion().contributes_to_gh())
            .copied()
            .collect();

        let kh_solutions = solve_single_target(self.target_kh, &kh_concs);
        let gh_solutions = solve_single_target(self.target_gh, &gh_concs);

        // Cross-product
        let mut results: Vec<RecipeSolution> = Vec::new();

        match (kh_solutions.is_empty(), gh_solutions.is_empty()) {
            (true, true) => {}
            (false, true) => {
                for kh_sol in &kh_solutions {
                    let sol = build_solution(kh_sol, &[], &self.concentrates);
                    results.push(sol);
                }
            }
            (true, false) => {
                for gh_sol in &gh_solutions {
                    let sol = build_solution(&[], gh_sol, &self.concentrates);
                    results.push(sol);
                }
            }
            (false, false) => {
                for kh_sol in &kh_solutions {
                    for gh_sol in &gh_solutions {
                        let sol = build_solution(kh_sol, gh_sol, &self.concentrates);
                        results.push(sol);
                    }
                }
            }
        }

        // Sort: fewest concentrates first
        results.sort_by_key(|s| s.usages.len());
        results
    }

    // -----------------------------------------------------------------------
    // Solver — Mode B: manual ions
    // -----------------------------------------------------------------------

    /// For each target ion, find the concentrate(s) that supply it and
    /// calculate mL needed. Returns one solution (not ranked — additive).
    /// Respects `manual_ion_filter`: if non-empty, only uses selected concentrates.
    pub fn solve_manual(&self) -> RecipeSolution {
        let targets: &[(Ion, f64)] = &[
            (Ion::Ca, self.target_ca),
            (Ion::Mg, self.target_mg),
            (Ion::HCO3, self.target_hco3),
            (Ion::Na, self.target_na),
            (Ion::K, self.target_k),
        ];

        let mut usages: Vec<ConcentrateUsage> = Vec::new();

        // Determine available concentrates based on filter
        let use_filter = !self.manual_ion_filter.is_empty();

        for (ion, target_ppm) in targets {
            if *target_ppm <= 0.0 {
                continue;
            }
            // Find concentrates whose primary ion matches and are in filter (if filter is set)
            let matching: Vec<&Concentrate> = self
                .concentrates
                .iter()
                .filter(|c| {
                    c.primary_ion() == *ion
                        && (!use_filter || self.manual_ion_filter.contains(&c.id))
                })
                .collect();

            if matching.is_empty() {
                continue;
            }

            if matching.len() == 1 {
                let c = matching[0];
                let ml = ml_needed(*target_ppm, c.primary_ppm_per_ml());
                usages.push(ConcentrateUsage {
                    concentrate_id: c.id,
                    ml_per_liter: ml,
                });
            } else {
                // Multiple concentrates for the same ion — proportional split
                // (equal contribution each)
                let n = matching.len() as f64;
                for c in &matching {
                    let ml = ml_needed(*target_ppm / n, c.primary_ppm_per_ml());
                    usages.push(ConcentrateUsage {
                        concentrate_id: c.id,
                        ml_per_liter: ml,
                    });
                }
            }
        }

        build_solution_from_usages(&usages, &self.concentrates)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// For a single-axis target (either KH or GH), return lists of usages
/// (one or two concentrates) that hit the target.
fn solve_single_target<'a>(
    target: f64,
    concs: &[&'a Concentrate],
) -> Vec<Vec<ConcentrateUsage>> {
    let mut solutions: Vec<Vec<ConcentrateUsage>> = Vec::new();

    if target <= 0.0 {
        // Target is zero — no concentrate needed
        solutions.push(vec![]);
        return solutions;
    }

    if concs.is_empty() {
        return solutions;
    }

    // Single-concentrate solutions
    for c in concs.iter() {
        let ml = ml_needed(target, c.primary_ppm_per_ml());
        if ml > 0.0 && ml.is_finite() {
            solutions.push(vec![ConcentrateUsage {
                concentrate_id: c.id,
                ml_per_liter: ml,
            }]);
        }
    }

    // Two-concentrate solutions (split target 50/50, 75/25, 25/75)
    for i in 0..concs.len() {
        for j in (i + 1)..concs.len() {
            for &ratio in &[0.5_f64, 0.75, 0.25] {
                let ml_i = ml_needed(target * ratio, concs[i].primary_ppm_per_ml());
                let ml_j = ml_needed(target * (1.0 - ratio), concs[j].primary_ppm_per_ml());
                if ml_i > 0.0 && ml_j > 0.0 && ml_i.is_finite() && ml_j.is_finite() {
                    solutions.push(vec![
                        ConcentrateUsage {
                            concentrate_id: concs[i].id,
                            ml_per_liter: ml_i,
                        },
                        ConcentrateUsage {
                            concentrate_id: concs[j].id,
                            ml_per_liter: ml_j,
                        },
                    ]);
                }
            }
        }
    }

    solutions
}

fn build_solution(
    kh_usages: &[ConcentrateUsage],
    gh_usages: &[ConcentrateUsage],
    all_concs: &[Concentrate],
) -> RecipeSolution {
    let mut usages = Vec::new();
    usages.extend_from_slice(kh_usages);
    usages.extend_from_slice(gh_usages);
    build_solution_from_usages(&usages, all_concs)
}

fn build_solution_from_usages(
    usages: &[ConcentrateUsage],
    all_concs: &[Concentrate],
) -> RecipeSolution {
    // Accumulate ion contributions
    use std::collections::HashMap;
    let mut ion_map: HashMap<Ion, (f64, f64)> = HashMap::new(); // (ppm_caco3, mg_per_l)

    for usage in usages {
        if let Some(conc) = all_concs.iter().find(|c| c.id == usage.concentrate_id) {
            let profile = conc.ion_profile_per_ml();
            for (ion, ppm, mg) in profile {
                let entry = ion_map.entry(ion).or_insert((0.0, 0.0));
                entry.0 += ppm * usage.ml_per_liter;
                entry.1 += mg * usage.ml_per_liter;
            }
        }
    }

    let mut ions: Vec<(Ion, f64, f64)> = ion_map
        .into_iter()
        .map(|(ion, (ppm, mg))| (ion, ppm, mg))
        .collect();

    // Stable display order
    let order = [Ion::Ca, Ion::Mg, Ion::HCO3, Ion::Na, Ion::K, Ion::Cl, Ion::SO4];
    ions.sort_by_key(|(ion, _, _)| order.iter().position(|o| o == ion).unwrap_or(99));

    let kh = ions
        .iter()
        .filter(|(ion, _, _)| ion.contributes_to_kh())
        .map(|(_, ppm, _)| ppm)
        .sum();
    let gh = ions
        .iter()
        .filter(|(ion, _, _)| ion.contributes_to_gh())
        .map(|(_, ppm, _)| ppm)
        .sum();

    RecipeSolution {
        usages: usages.to_vec(),
        ions,
        kh,
        gh,
    }
}
