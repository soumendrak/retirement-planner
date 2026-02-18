use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Income source (salary, pension, rental, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Income {
    pub id: Uuid,
    pub name: String,
    /// Monthly gross income in cents/paise
    pub monthly_amount_cents: i64,
    /// Year this income starts (0 = from current age)
    pub start_year: u32,
    /// Year this income ends (None = indefinite)
    pub end_year: Option<u32>,
    /// Annual growth rate as percentage (e.g., 3.0 means 3% annual increase)
    pub annual_growth: f64,
}

impl Income {
    pub fn new(name: String, monthly_amount_cents: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            monthly_amount_cents,
            start_year: 0,
            end_year: None,
            annual_growth: 0.0,
        }
    }

    /// Calculate monthly income for a given year.
    /// Returns None if the income is not active in that year.
    pub fn monthly_for_year(&self, year: u32) -> Option<i64> {
        if year < self.start_year {
            return None;
        }
        if let Some(end) = self.end_year {
            if year > end {
                return None;
            }
        }

        let growth_factor = (1.0 + self.annual_growth / 100.0).powi(year as i32);
        Some((self.monthly_amount_cents as f64 * growth_factor) as i64)
    }
}
