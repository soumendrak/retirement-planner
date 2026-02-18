use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::currency::Currency;

/// User profile containing personal retirement parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub current_age: u8,
    pub retirement_age: u8,
    pub life_expectancy: u8,
    pub base_currency: Currency,
    /// User-supplied USD↔INR exchange rate (e.g., 83.0 means 1 USD = 83 INR)
    pub exchange_rate: f64,
    /// Flat effective tax rate as percentage (e.g., 25.0 means 25%)
    pub effective_tax_rate: f64,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            current_age: 30,
            retirement_age: 60,
            life_expectancy: 85,
            base_currency: Currency::USD,
            exchange_rate: 83.0,
            effective_tax_rate: 25.0,
        }
    }

    /// Returns the number of years until retirement.
    pub fn years_to_retirement(&self) -> u8 {
        self.retirement_age.saturating_sub(self.current_age)
    }

    /// Returns the number of years in retirement.
    pub fn years_in_retirement(&self) -> u8 {
        self.life_expectancy.saturating_sub(self.retirement_age)
    }
}
