/// All ions tracked by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ion {
    Ca,
    Mg,
    HCO3,
    Na,
    K,
    Cl,
    SO4,
}

impl Ion {
    pub fn label(&self) -> &'static str {
        match self {
            Ion::Ca => "Ca²⁺",
            Ion::Mg => "Mg²⁺",
            Ion::HCO3 => "HCO₃⁻",
            Ion::Na => "Na⁺",
            Ion::K => "K⁺",
            Ion::Cl => "Cl⁻",
            Ion::SO4 => "SO₄²⁻",
        }
    }

    /// Atomic / formula weight in g/mol.
    pub fn molar_mass(&self) -> f64 {
        match self {
            Ion::Ca => 40.078,
            Ion::Mg => 24.305,
            Ion::HCO3 => 61.017,
            Ion::Na => 22.990,
            Ion::K => 39.098,
            Ion::Cl => 35.453,
            Ion::SO4 => 96.062,
        }
    }

    /// Charge magnitude (valence).
    pub fn valence(&self) -> f64 {
        match self {
            Ion::Ca | Ion::Mg | Ion::SO4 => 2.0,
            Ion::HCO3 | Ion::Na | Ion::K | Ion::Cl => 1.0,
        }
    }

    /// Equivalent weight = molar_mass / valence.
    pub fn equivalent_weight(&self) -> f64 {
        self.molar_mass() / self.valence()
    }

    /// Conversion factor: (mg/L of ion) × factor = ppm-as-CaCO₃.
    /// factor = 50 / equivalent_weight
    pub fn ppm_caco3_factor(&self) -> f64 {
        50.0 / self.equivalent_weight()
    }

    /// Whether this ion contributes to GH (general hardness).
    pub fn contributes_to_gh(&self) -> bool {
        matches!(self, Ion::Ca | Ion::Mg)
    }

    /// Whether this ion contributes to KH (carbonate hardness / alkalinity).
    pub fn contributes_to_kh(&self) -> bool {
        matches!(self, Ion::HCO3)
    }
}

/// Supported salts / compounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compound {
    CaCl2,
    MgSO4,
    MgCl2,
    NaHCO3,
    KHCO3,
    NaCl,
}

impl Compound {
    pub const ALL: &'static [Compound] = &[
        Compound::CaCl2,
        Compound::MgSO4,
        Compound::MgCl2,
        Compound::NaHCO3,
        Compound::KHCO3,
        Compound::NaCl,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Compound::CaCl2 => "Calcium Chloride",
            Compound::MgSO4 => "Magnesium Sulfate (Epsom)",
            Compound::MgCl2 => "Magnesium Chloride",
            Compound::NaHCO3 => "Sodium Bicarbonate (Baking Soda)",
            Compound::KHCO3 => "Potassium Bicarbonate",
            Compound::NaCl => "Sodium Chloride (Salt)",
        }
    }

    pub fn formula_anhydrous(&self) -> &'static str {
        match self {
            Compound::CaCl2 => "CaCl₂",
            Compound::MgSO4 => "MgSO₄",
            Compound::MgCl2 => "MgCl₂",
            Compound::NaHCO3 => "NaHCO₃",
            Compound::KHCO3 => "KHCO₃",
            Compound::NaCl => "NaCl",
        }
    }

    /// Hydrated form label (e.g. "·2H₂O"), if applicable.
    pub fn hydration_label(&self) -> Option<&'static str> {
        match self {
            Compound::CaCl2 => Some("·2H₂O"),
            Compound::MgSO4 => Some("·7H₂O"),
            Compound::MgCl2 => Some("·6H₂O"),
            _ => None,
        }
    }

    pub fn has_hydrated_form(&self) -> bool {
        self.hydration_label().is_some()
    }

    /// Molar mass of the anhydrous form (g/mol).
    pub fn mw_anhydrous(&self) -> f64 {
        match self {
            Compound::CaCl2 => 110.984,
            Compound::MgSO4 => 120.366,
            Compound::MgCl2 => 95.211,
            Compound::NaHCO3 => 84.007,
            Compound::KHCO3 => 100.115,
            Compound::NaCl => 58.443,
        }
    }

    /// Molar mass of the hydrated form (g/mol), or anhydrous if no hydrate.
    pub fn mw_hydrated(&self) -> f64 {
        match self {
            Compound::CaCl2 => 147.015, // +2×18.015
            Compound::MgSO4 => 246.473, // +7×18.015
            Compound::MgCl2 => 203.301, // +6×18.015
            _ => self.mw_anhydrous(),
        }
    }

    pub fn mw(&self, use_anhydrous: bool) -> f64 {
        if use_anhydrous {
            self.mw_anhydrous()
        } else {
            self.mw_hydrated()
        }
    }

    /// Stoichiometric ion yields: (Ion, moles of ion per mole of compound).
    pub fn ions(&self) -> &'static [(Ion, f64)] {
        match self {
            Compound::CaCl2 => &[(Ion::Ca, 1.0), (Ion::Cl, 2.0)],
            Compound::MgSO4 => &[(Ion::Mg, 1.0), (Ion::SO4, 1.0)],
            Compound::MgCl2 => &[(Ion::Mg, 1.0), (Ion::Cl, 2.0)],
            Compound::NaHCO3 => &[(Ion::Na, 1.0), (Ion::HCO3, 1.0)],
            Compound::KHCO3 => &[(Ion::K, 1.0), (Ion::HCO3, 1.0)],
            Compound::NaCl => &[(Ion::Na, 1.0), (Ion::Cl, 1.0)],
        }
    }

    /// The ion whose ppm-as-CaCO₃ is used to define concentrate potency.
    pub fn primary_ion(&self) -> Ion {
        match self {
            Compound::CaCl2 => Ion::Ca,
            Compound::MgSO4 => Ion::Mg,
            Compound::MgCl2 => Ion::Mg,
            Compound::NaHCO3 => Ion::HCO3,
            Compound::KHCO3 => Ion::HCO3,
            Compound::NaCl => Ion::Na,
        }
    }

    /// What the compound primarily affects (for UI grouping).
    pub fn affects_gh(&self) -> bool {
        self.primary_ion().contributes_to_gh()
    }

    pub fn affects_kh(&self) -> bool {
        self.primary_ion().contributes_to_kh()
    }

    /// Human-readable description of what this compound affects.
    pub fn affects_label(&self) -> &'static str {
        match self {
            Compound::CaCl2 => "GH (Ca²⁺)",
            Compound::MgSO4 => "GH (Mg²⁺)",
            Compound::MgCl2 => "GH (Mg²⁺)",
            Compound::NaHCO3 => "KH",
            Compound::KHCO3 => "KH",
            Compound::NaCl => "Salinity only",
        }
    }
}
