use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Expense, Income, Instrument, OneTimeEvent};

/// Inflation configuration with per-category and blanket options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationConfig {
    /// Default blanket inflation rate as percentage
    pub blanket_rate: f64,
    /// If true, ignore per-category rates and use blanket_rate for all
    pub use_blanket: bool,
}

impl Default for InflationConfig {
    fn default() -> Self {
        Self {
            blanket_rate: 3.0,
            use_blanket: false,
        }
    }
}

/// A complete what-if retirement scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub profile_id: Uuid,
    pub instruments: Vec<Instrument>,
    pub expenses: Vec<Expense>,
    pub incomes: Vec<Income>,
    pub inflation_config: InflationConfig,
    pub notes: String,
    /// One-time lump-sum events (inheritance, home purchase, etc.)
    #[serde(default)]
    pub one_time_events: Vec<OneTimeEvent>,
}

impl Scenario {
    pub fn new(name: String, profile_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            profile_id,
            instruments: Vec::new(),
            expenses: Vec::new(),
            incomes: Vec::new(),
            inflation_config: InflationConfig::default(),
            notes: String::new(),
            one_time_events: Vec::new(),
        }
    }

    /// Calculate total current corpus across all instruments.
    pub fn total_corpus_cents(&self) -> i64 {
        self.instruments.iter().map(|i| i.current_value_cents).sum()
    }

    /// Calculate total monthly contributions.
    pub fn total_monthly_contributions_cents(&self) -> i64 {
        self.instruments
            .iter()
            .map(|i| i.monthly_contribution_cents)
            .sum()
    }

    /// Calculate total monthly expenses.
    pub fn total_monthly_expenses_cents(&self) -> i64 {
        self.expenses.iter().map(|e| e.monthly_amount_cents).sum()
    }

    /// Calculate weighted average expected return across all instruments.
    pub fn weighted_average_return(&self) -> f64 {
        let total = self.total_corpus_cents() as f64;
        if total == 0.0 {
            return 0.0;
        }

        let weighted: f64 = self
            .instruments
            .iter()
            .map(|i| (i.current_value_cents as f64) * i.expected_annual_return)
            .sum();

        weighted / total
    }
}
