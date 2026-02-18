//! Retirement Planner Core
//!
//! Domain logic for retirement planning with zero GUI dependencies.
//! All monetary values stored as cents/paise (i64) for precision.

pub mod currency;
pub mod engine;
pub mod export;
pub mod models;
pub mod storage;

pub use currency::{format_currency, Cents, Currency};
pub use engine::{projection, projection_with_band, ReturnBand, YearProjection};
pub use export::PlanExport;
pub use models::*;
