use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A one-time lump-sum financial event at a specific age.
///
/// Positive amounts are inflows (inheritance, bonus, property sale).
/// Negative amounts are outflows (home purchase, medical, education).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTimeEvent {
    pub id: Uuid,
    pub name: String,
    /// Age at which this event occurs
    pub age: u8,
    /// Amount in cents/paise — positive = inflow, negative = outflow
    pub amount_cents: i64,
    pub notes: String,
}

impl OneTimeEvent {
    pub fn new(name: String, age: u8, amount_cents: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            age,
            amount_cents,
            notes: String::new(),
        }
    }

    /// Whether this event occurs in the given simulation year.
    pub fn occurs_at_year(&self, current_age: u8, year: u32) -> bool {
        let event_year = self.age.saturating_sub(current_age) as u32;
        year == event_year
    }
}
