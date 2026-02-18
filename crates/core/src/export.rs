//! JSON export/import for retirement plans.
//!
//! Bundles a profile with all its data into a single JSON-serializable struct.

use serde::{Deserialize, Serialize};

use crate::engine::YearProjection;
use crate::models::{Expense, Income, InflationConfig, Instrument, OneTimeEvent, Profile};

/// A complete retirement plan snapshot for JSON export/import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExport {
    /// Format version for forward compatibility
    pub version: u32,
    pub profile: Profile,
    pub instruments: Vec<Instrument>,
    pub expenses: Vec<Expense>,
    pub incomes: Vec<Income>,
    pub one_time_events: Vec<OneTimeEvent>,
    pub inflation_config: InflationConfig,
    pub scenario_names: Vec<String>,
}

impl PlanExport {
    pub const CURRENT_VERSION: u32 = 1;

    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Basic validation of imported data.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.profile.name.is_empty() {
            errors.push("Profile name is empty".to_string());
        }
        if self.profile.current_age >= self.profile.retirement_age {
            errors.push("Current age must be less than retirement age".to_string());
        }
        if self.profile.retirement_age >= self.profile.life_expectancy {
            errors.push("Retirement age must be less than life expectancy".to_string());
        }
        if self.profile.effective_tax_rate < 0.0 || self.profile.effective_tax_rate > 100.0 {
            errors.push("Tax rate must be between 0 and 100".to_string());
        }

        for inst in &self.instruments {
            if inst.name.is_empty() {
                errors.push("Instrument has empty name".to_string());
            }
            if inst.current_value_cents < 0 {
                errors.push(format!("Instrument '{}' has negative value", inst.name));
            }
        }

        for exp in &self.expenses {
            if exp.monthly_amount_cents < 0 {
                errors.push(format!(
                    "Expense '{}' has negative amount",
                    exp.category.as_str()
                ));
            }
        }

        for inc in &self.incomes {
            if inc.name.is_empty() {
                errors.push("Income source has empty name".to_string());
            }
            if inc.monthly_amount_cents < 0 {
                errors.push(format!("Income '{}' has negative amount", inc.name));
            }
        }

        errors
    }
}

/// Export a projection table to CSV format.
///
/// Columns: Age, Year, Corpus, Income, Expenses, Withdrawal, Status
pub fn projections_to_csv(projections: &[YearProjection], currency_symbol: &str) -> String {
    let mut lines = vec![
        "Age,Year,Corpus,Annual Income,Annual Expenses,Net Withdrawal,Status".to_string(),
    ];

    for p in projections {
        let status = if !p.is_retired {
            "Working"
        } else if p.corpus_cents > 0 {
            "Retired"
        } else {
            "Depleted"
        };

        lines.push(format!(
            "{},{},{}{:.2},{}{:.2},{}{:.2},{}{:.2},{}",
            p.age,
            p.year,
            currency_symbol, p.corpus_cents as f64 / 100.0,
            currency_symbol, p.total_income_cents as f64 / 100.0,
            currency_symbol, p.total_expenses_cents as f64 / 100.0,
            currency_symbol, p.net_withdrawal_cents as f64 / 100.0,
            status,
        ));
    }

    lines.join("\n")
}
