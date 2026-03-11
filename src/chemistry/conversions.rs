use super::compounds::{Compound, Ion};

/// Given grams of compound dissolved to make `stock_volume_ml` mL of stock,
/// returns the mg/L of each ion in the stock solution.
///
/// Then when `concentrate_ml` of that stock is added to `final_volume_l` of
/// DI water, the final ion concentration (mg/L) is:
///   `mg_per_l_in_final = mg_per_l_in_stock * (concentrate_ml / 1000) / final_volume_l`
///
/// However, our concentrate model uses a simpler definition:
///   potency = ppm-as-CaCO3 of primary ion when 1 mL of stock is added to 1 L of water.
///
/// This file provides the math to convert between:
///   potency (ppm-as-CaCO3 / mL-into-1L)  ↔  grams of compound

/// Convert potency (ppm-as-CaCO3 of primary ion, per 1 mL into 1 L) to
/// the required grams of compound to make `stock_volume_ml` mL of stock.
///
/// Derivation:
///   1 mL into 1 L  →  dilution factor = 1/1000
///   mg/L primary ion in final water = potency / ion.ppm_caco3_factor()
///   mg/L primary ion in stock = mg/L_final / (1/1000) = mg/L_final × 1000
///   mg primary ion in `stock_volume_ml` mL = mg/L_stock × (stock_volume_ml / 1000)
///   moles of primary ion = mg_total / (ion.molar_mass() × 1000)   [g→mg]
///   moles of compound = moles_of_ion / stoichiometric_ratio
///   grams of compound = moles_compound × compound.mw(use_anhydrous)
pub fn potency_to_grams(
    compound: Compound,
    potency_ppm_caco3_per_ml: f64,
    stock_volume_ml: f64,
    use_anhydrous: bool,
) -> f64 {
    if potency_ppm_caco3_per_ml <= 0.0 || stock_volume_ml <= 0.0 {
        return 0.0;
    }

    let primary_ion = compound.primary_ion();
    let stoich = stoich_ratio(compound, primary_ion);

    // mg/L of primary ion in the final 1 L water (when 1 mL of stock is added)
    let mg_per_l_final = potency_ppm_caco3_per_ml / primary_ion.ppm_caco3_factor();

    // mg/L of primary ion in the stock (dilution 1 mL into 1000 mL total ≈ 1 L)
    let mg_per_l_stock = mg_per_l_final * 1000.0;

    // total mg of primary ion in the entire stock volume
    let mg_ion_total = mg_per_l_stock * (stock_volume_ml / 1000.0);

    // moles of primary ion
    let mol_ion = mg_ion_total / (primary_ion.molar_mass() * 1000.0);

    // moles of compound (accounting for stoichiometry)
    let mol_compound = mol_ion / stoich;

    // grams of compound
    mol_compound * compound.mw(use_anhydrous)
}

/// Stoichiometric ratio: moles of `ion` per mole of `compound`.
pub fn stoich_ratio(compound: Compound, ion: Ion) -> f64 {
    compound
        .ions()
        .iter()
        .find(|(i, _)| *i == ion)
        .map(|(_, r)| *r)
        .unwrap_or(0.0)
}

/// Full ion profile when 1 mL of a concentrate (built with the given potency
/// for the primary ion) is added to 1 L of water.
///
/// Returns a list of (Ion, ppm-as-CaCO3, mg/L).
pub fn ion_profile_per_ml(
    compound: Compound,
    potency_ppm_caco3_per_ml: f64,
) -> Vec<(Ion, f64, f64)> {
    if potency_ppm_caco3_per_ml <= 0.0 {
        return vec![];
    }

    let primary_ion = compound.primary_ion();
    let primary_stoich = stoich_ratio(compound, primary_ion);

    // mg/L of primary ion in final water (1 mL concentrate into 1 L)
    let mg_per_l_primary = potency_ppm_caco3_per_ml / primary_ion.ppm_caco3_factor();

    // moles of primary ion per litre of final water
    let mol_primary = mg_per_l_primary / (primary_ion.molar_mass() * 1000.0);

    // moles of compound that produced those moles of primary ion
    let mol_compound = mol_primary / primary_stoich;

    compound
        .ions()
        .iter()
        .map(|(ion, stoich)| {
            let mol_ion = mol_compound * stoich;
            let mg_per_l = mol_ion * ion.molar_mass() * 1000.0;
            let ppm_caco3 = mg_per_l * ion.ppm_caco3_factor();
            (*ion, ppm_caco3, mg_per_l)
        })
        .collect()
}

/// Compute KH and GH (both in ppm-as-CaCO3) from a flat ion map.
/// `ions`: list of (Ion, ppm-as-CaCO3).
pub fn compute_kh_gh(ions: &[(Ion, f64)]) -> (f64, f64) {
    let kh = ions
        .iter()
        .filter(|(ion, _)| ion.contributes_to_kh())
        .map(|(_, ppm)| ppm)
        .sum();
    let gh = ions
        .iter()
        .filter(|(ion, _)| ion.contributes_to_gh())
        .map(|(_, ppm)| ppm)
        .sum();
    (kh, gh)
}

/// Given a target ppm-as-CaCO3 for a specific ion and a concentrate's potency,
/// return the mL of concentrate to add to 1 L of final water.
pub fn ml_needed(target_ppm_caco3: f64, potency_ppm_caco3_per_ml: f64) -> f64 {
    if potency_ppm_caco3_per_ml <= 0.0 {
        return 0.0;
    }
    target_ppm_caco3 / potency_ppm_caco3_per_ml
}
