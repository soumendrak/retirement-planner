use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Expense category with support for custom categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpenseCategory {
    Housing,
    Healthcare,
    Food,
    Transport,
    Leisure,
    Other(String),
}

impl ExpenseCategory {
    pub fn as_str(&self) -> &str {
        match self {
            ExpenseCategory::Housing => "Housing",
            ExpenseCategory::Healthcare => "Healthcare",
            ExpenseCategory::Food => "Food",
            ExpenseCategory::Transport => "Transport",
            ExpenseCategory::Leisure => "Leisure",
            ExpenseCategory::Other(s) => s,
        }
    }

    /// Returns all predefined categories.
    pub fn predefined() -> &'static [ExpenseCategory] {
        &[
            ExpenseCategory::Housing,
            ExpenseCategory::Healthcare,
            ExpenseCategory::Food,
            ExpenseCategory::Transport,
            ExpenseCategory::Leisure,
        ]
    }
}

impl std::fmt::Display for ExpenseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A recurring monthly expense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: Uuid,
    pub category: ExpenseCategory,
    /// Monthly amount in cents/paise
    pub monthly_amount_cents: i64,
    /// Per-category inflation rate as percentage.
    /// None means use the blanket inflation rate.
    pub inflation_rate: Option<f64>,
}

impl Expense {
    pub fn new(category: ExpenseCategory, monthly_amount_cents: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            monthly_amount_cents,
            inflation_rate: None,
        }
    }

    /// Calculate the annual expense for a given year, applying inflation.
    /// Returns monthly amount in cents for that year.
    pub fn monthly_for_year(&self, year: u32, blanket_rate: f64) -> i64 {
        let rate = self.inflation_rate.unwrap_or(blanket_rate) / 100.0;
        let inflation_factor = (1.0 + rate).powi(year as i32);
        (self.monthly_amount_cents as f64 * inflation_factor) as i64
    }
}
