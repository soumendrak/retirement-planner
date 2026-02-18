use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Asset class for an investment instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    Equity,
    Debt,
    RealEstate,
    Cash,
    Other,
}

impl AssetClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetClass::Equity => "Equity",
            AssetClass::Debt => "Debt",
            AssetClass::RealEstate => "Real Estate",
            AssetClass::Cash => "Cash",
            AssetClass::Other => "Other",
        }
    }
}

impl std::fmt::Display for AssetClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AssetClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "equity" => Ok(AssetClass::Equity),
            "debt" => Ok(AssetClass::Debt),
            "real estate" | "realestate" | "real-estate" => Ok(AssetClass::RealEstate),
            "cash" => Ok(AssetClass::Cash),
            "other" => Ok(AssetClass::Other),
            _ => Err(format!("Unknown asset class: {}", s)),
        }
    }
}

/// An investment instrument (e.g., stock, mutual fund, bond).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: Uuid,
    pub name: String,
    pub asset_class: AssetClass,
    /// Expected annual return as percentage (e.g., 8.0 means 8%)
    pub expected_annual_return: f64,
    /// Current value in cents/paise
    pub current_value_cents: i64,
    /// Monthly contribution in cents/paise
    pub monthly_contribution_cents: i64,
}

impl Instrument {
    pub fn new(name: String, asset_class: AssetClass) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            asset_class,
            expected_annual_return: 7.0,
            current_value_cents: 0,
            monthly_contribution_cents: 0,
        }
    }

    /// Apply one year of growth and contributions.
    /// Returns the new value in cents.
    pub fn project_one_year(&self, include_contributions: bool) -> i64 {
        let growth_factor = 1.0 + self.expected_annual_return / 100.0;
        let grown = (self.current_value_cents as f64 * growth_factor) as i64;

        if include_contributions {
            // Add 12 months of contributions
            grown + (self.monthly_contribution_cents * 12)
        } else {
            grown
        }
    }
}
