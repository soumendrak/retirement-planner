//! Projection engine for year-by-year retirement simulation.
//!
//! The engine runs a deterministic simulation from current age to life expectancy,
//! tracking corpus growth, withdrawals, and depletion.

use serde::{Deserialize, Serialize};

use crate::models::{Expense, Income, Instrument, Profile, Scenario};

/// Return band for stress-testing projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReturnBand {
    /// Returns reduced by 40% — pessimistic scenario
    Pessimistic,
    /// Returns as specified — the default
    #[default]
    Base,
    /// Returns increased by 40% — optimistic scenario
    Optimistic,
}

impl ReturnBand {
    /// Multiplier applied to each instrument's expected_annual_return.
    pub fn multiplier(self) -> f64 {
        match self {
            ReturnBand::Pessimistic => 0.60,
            ReturnBand::Base => 1.00,
            ReturnBand::Optimistic => 1.40,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ReturnBand::Pessimistic => "Pessimistic",
            ReturnBand::Base => "Base",
            ReturnBand::Optimistic => "Optimistic",
        }
    }
}

/// A single year's projection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearProjection {
    /// Age at the end of this year
    pub age: u8,
    /// Year number (0 = current year)
    pub year: u32,
    /// Corpus balance at end of year (cents)
    pub corpus_cents: i64,
    /// Total income for the year (cents)
    pub total_income_cents: i64,
    /// Total expenses for the year (cents)
    pub total_expenses_cents: i64,
    /// Net withdrawal from corpus (cents, positive = withdrawal)
    pub net_withdrawal_cents: i64,
    /// Whether this year is post-retirement
    pub is_retired: bool,
}

/// Summary of a scenario projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionSummary {
    /// Corpus at retirement age (cents)
    pub corpus_at_retirement_cents: i64,
    /// Age when corpus is depleted (None = never depletes)
    pub depletion_age: Option<u8>,
    /// Number of years corpus lasts in retirement
    pub years_corpus_lasts: Option<u32>,
    /// Safe monthly withdrawal rate (cents)
    pub safe_monthly_withdrawal_cents: i64,
    /// Whether the corpus never depletes
    pub corpus_never_depletes: bool,
}

/// Run a full projection for a scenario using the base return band.
///
/// # Arguments
/// * `profile` - User profile with age and retirement info
/// * `scenario` - Scenario with instruments, expenses, incomes, one-time events
///
/// # Returns
/// Vector of year projections from current age to life expectancy
pub fn projection(profile: &Profile, scenario: &Scenario) -> Vec<YearProjection> {
    projection_with_band(profile, scenario, ReturnBand::Base)
}

/// Run a full projection with a specific return band multiplier.
///
/// Use this to generate optimistic / base / pessimistic overlays.
pub fn projection_with_band(
    profile: &Profile,
    scenario: &Scenario,
    band: ReturnBand,
) -> Vec<YearProjection> {
    let mut results = Vec::new();
    let mut current_instruments: Vec<Instrument> = scenario.instruments.clone();
    let band_mult = band.multiplier();

    let total_years = profile
        .life_expectancy
        .saturating_sub(profile.current_age) as u32;

    for year in 0..=total_years {
        let age = profile.current_age.saturating_add(year as u8);
        let is_retired = age >= profile.retirement_age;

        // 1. Apply one-time events this year (before growth so event capital grows too)
        let one_time_net_cents: i64 = scenario
            .one_time_events
            .iter()
            .filter(|e| e.occurs_at_year(profile.current_age, year))
            .map(|e| e.amount_cents)
            .sum();

        if one_time_net_cents != 0 {
            apply_one_time_event(&mut current_instruments, one_time_net_cents);
        }

        // 2. Calculate total income for this year
        let total_income_cents = calculate_yearly_income(&scenario.incomes, year);

        // 3. Calculate total expenses for this year
        let total_expenses_cents =
            calculate_yearly_expenses(&scenario.expenses, year, &scenario.inflation_config);

        // 4. Grow instruments with band multiplier
        for inst in &mut current_instruments {
            let effective_return = inst.expected_annual_return * band_mult;
            let growth_factor = 1.0 + effective_return / 100.0;
            inst.current_value_cents =
                (inst.current_value_cents as f64 * growth_factor) as i64;

            if !is_retired {
                inst.current_value_cents = inst
                    .current_value_cents
                    .saturating_add(inst.monthly_contribution_cents.saturating_mul(12));
            }
        }

        // 5. Calculate corpus before withdrawal
        let corpus_cents: i64 = current_instruments
            .iter()
            .map(|i| i.current_value_cents)
            .sum::<i64>()
            .max(0);

        // 6. Handle withdrawals in retirement
        let net_withdrawal_cents = if is_retired {
            let needed = total_expenses_cents.saturating_sub(total_income_cents).max(0);

            let gross_withdrawal = if profile.effective_tax_rate > 0.0 {
                let tax_factor = 1.0 + profile.effective_tax_rate / 100.0;
                (needed as f64 * tax_factor) as i64
            } else {
                needed
            };

            if corpus_cents > 0 && gross_withdrawal > 0 {
                withdraw_from_instruments(&mut current_instruments, gross_withdrawal);
            }

            gross_withdrawal
        } else {
            0
        };

        // 7. Final corpus after withdrawal (floor at 0 for display)
        let final_corpus: i64 = current_instruments
            .iter()
            .map(|i| i.current_value_cents)
            .sum::<i64>()
            .max(0);

        results.push(YearProjection {
            age,
            year,
            corpus_cents: final_corpus,
            total_income_cents,
            total_expenses_cents,
            net_withdrawal_cents,
            is_retired,
        });

        if final_corpus <= 0 && year < total_years {
            for remaining_year in (year + 1)..=total_years {
                results.push(YearProjection {
                    age: profile.current_age.saturating_add(remaining_year as u8),
                    year: remaining_year,
                    corpus_cents: 0,
                    total_income_cents: 0,
                    total_expenses_cents: calculate_yearly_expenses(
                        &scenario.expenses,
                        remaining_year,
                        &scenario.inflation_config,
                    ),
                    net_withdrawal_cents: 0,
                    is_retired: true,
                });
            }
            break;
        }
    }

    results
}

/// Calculate summary from projection results.
pub fn summarize_projection(
    profile: &Profile,
    projections: &[YearProjection],
) -> ProjectionSummary {
    // Find corpus at retirement
    let corpus_at_retirement_cents = projections
        .iter()
        .find(|p| p.age == profile.retirement_age)
        .map(|p| p.corpus_cents)
        .unwrap_or(0);

    // Find depletion age
    let depletion_age = projections
        .iter()
        .find(|p| p.corpus_cents <= 0 && p.is_retired)
        .map(|p| p.age);

    // Calculate years corpus lasts
    let years_corpus_lasts = depletion_age.map(|age| {
        (age as u32).saturating_sub(profile.retirement_age as u32)
    });

    // Calculate safe monthly withdrawal (4% rule approximation)
    // SWR = corpus_at_retirement * 0.04 / 12
    let safe_monthly_withdrawal_cents = (corpus_at_retirement_cents as f64 * 0.04 / 12.0) as i64;

    // Check if corpus never depletes
    let corpus_never_depletes = projections
        .last()
        .map(|p| p.corpus_cents > 0)
        .unwrap_or(false);

    ProjectionSummary {
        corpus_at_retirement_cents,
        depletion_age,
        years_corpus_lasts,
        safe_monthly_withdrawal_cents,
        corpus_never_depletes,
    }
}

/// Calculate total income for a given year.
fn calculate_yearly_income(incomes: &[Income], year: u32) -> i64 {
    incomes
        .iter()
        .filter_map(|inc| inc.monthly_for_year(year))
        .map(|monthly| monthly * 12)
        .sum()
}

/// Calculate total expenses for a given year.
fn calculate_yearly_expenses(
    expenses: &[Expense],
    year: u32,
    config: &crate::models::InflationConfig,
) -> i64 {
    let blanket_rate = if config.use_blanket {
        config.blanket_rate
    } else {
        0.0 // Will use per-category rates
    };

    expenses
        .iter()
        .map(|exp| {
            let monthly = if config.use_blanket {
                exp.monthly_for_year(year, blanket_rate)
            } else {
                exp.monthly_for_year(year, config.blanket_rate)
            };
            monthly * 12
        })
        .sum()
}

/// Apply a one-time lump-sum event to the instrument pool.
///
/// Positive amount: distribute inflow proportionally to existing balances (or evenly if all zero).
/// Negative amount: withdraw from instruments proportionally (same as withdrawal).
fn apply_one_time_event(instruments: &mut [Instrument], net_cents: i64) {
    if net_cents > 0 {
        // Inflow: add to first instrument (or spread proportionally)
        let total: i64 = instruments.iter().map(|i| i.current_value_cents).sum();
        if total <= 0 || instruments.is_empty() {
            if let Some(first) = instruments.first_mut() {
                first.current_value_cents = first.current_value_cents.saturating_add(net_cents);
            }
        } else {
            for inst in instruments.iter_mut() {
                let share = (net_cents as f64 * inst.current_value_cents as f64 / total as f64) as i64;
                inst.current_value_cents = inst.current_value_cents.saturating_add(share);
            }
        }
    } else {
        withdraw_from_instruments(instruments, net_cents.unsigned_abs() as i64);
    }
}

/// Withdraw an amount from instruments proportionally.
fn withdraw_from_instruments(instruments: &mut [Instrument], amount_cents: i64) {
    let total: i64 = instruments.iter().map(|i| i.current_value_cents).sum();
    if total == 0 {
        return;
    }

    let mut remaining = amount_cents;

    for inst in instruments.iter_mut() {
        if remaining <= 0 {
            break;
        }

        let proportion = inst.current_value_cents as f64 / total as f64;
        let withdraw = ((amount_cents as f64 * proportion) as i64).min(inst.current_value_cents);
        inst.current_value_cents -= withdraw;
        remaining -= withdraw;
    }

    // If still remaining, take from any positive balances
    for inst in instruments.iter_mut() {
        if remaining <= 0 {
            break;
        }
        let withdraw = remaining.min(inst.current_value_cents);
        inst.current_value_cents -= withdraw;
        remaining -= withdraw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, ExpenseCategory, InflationConfig};

    fn test_profile() -> Profile {
        Profile {
            id: uuid::Uuid::nil(),
            name: "Test".to_string(),
            current_age: 30,
            retirement_age: 60,
            life_expectancy: 85,
            base_currency: crate::currency::Currency::USD,
            exchange_rate: 83.0,
            effective_tax_rate: 25.0,
        }
    }

    fn test_scenario() -> Scenario {
        Scenario {
            id: uuid::Uuid::nil(),
            name: "Test".to_string(),
            profile_id: uuid::Uuid::nil(),
            instruments: vec![Instrument {
                id: uuid::Uuid::nil(),
                name: "Equity".to_string(),
                asset_class: AssetClass::Equity,
                expected_annual_return: 7.0,
                current_value_cents: 10_000_000, // $100k in cents
                monthly_contribution_cents: 100_000, // $1k/month in cents
            }],
            expenses: vec![Expense {
                id: uuid::Uuid::nil(),
                category: ExpenseCategory::Housing,
                monthly_amount_cents: 200_000, // $2k/month in cents
                inflation_rate: Some(3.0),
            }],
            incomes: vec![Income {
                id: uuid::Uuid::nil(),
                name: "Salary".to_string(),
                monthly_amount_cents: 500_000, // $5k/month in cents
                start_year: 0,
                end_year: Some(29), // Ends at retirement (30 years)
                annual_growth: 2.0,
            }],
            inflation_config: InflationConfig {
                blanket_rate: 3.0,
                use_blanket: false,
            },
            notes: String::new(),
            one_time_events: vec![],
        }
    }

    #[test]
    fn test_projection_runs() {
        let profile = test_profile();
        let scenario = test_scenario();
        let results = projection(&profile, &scenario);

        // Should have results from age 30 to 85
        assert_eq!(results.len(), 56);
        assert_eq!(results[0].age, 30);
        assert_eq!(results[0].year, 0);
        assert!(!results[0].is_retired);

        // First year corpus should grow
        assert!(results[0].corpus_cents > 10_000_000);
    }

    #[test]
    fn test_corpus_grows_pre_retirement() {
        let profile = test_profile();
        let scenario = test_scenario();
        let results = projection(&profile, &scenario);

        // Find retirement year
        let retirement_idx = results
            .iter()
            .position(|p| p.age == profile.retirement_age)
            .unwrap();

        // Corpus should be growing until retirement
        for i in 1..retirement_idx {
            assert!(
                results[i].corpus_cents >= results[i - 1].corpus_cents,
                "Corpus should grow pre-retirement at year {}",
                i
            );
        }
    }

    #[test]
    fn test_summary_calculation() {
        let profile = test_profile();
        let scenario = test_scenario();
        let results = projection(&profile, &scenario);
        let summary = summarize_projection(&profile, &results);

        // Should have corpus at retirement
        assert!(summary.corpus_at_retirement_cents > 0);

        // SWR should be reasonable (4% of corpus / 12)
        let expected_swr = (summary.corpus_at_retirement_cents as f64 * 0.04 / 12.0) as i64;
        assert_eq!(summary.safe_monthly_withdrawal_cents, expected_swr);
    }

    #[test]
    fn test_depletion_detection() {
        let mut profile = test_profile();
        profile.retirement_age = 31; // Retire very early
        profile.life_expectancy = 40;

        let mut scenario = test_scenario();
        scenario.instruments[0].current_value_cents = 1_000_000; // Only $10k in cents
        scenario.instruments[0].monthly_contribution_cents = 0;
        scenario.expenses[0].monthly_amount_cents = 500_000; // $5k/month in cents
        scenario.incomes.clear(); // No income at all

        let results = projection(&profile, &scenario);
        let summary = summarize_projection(&profile, &results);

        // Corpus should deplete: $10k can't sustain $60k/year expenses
        assert!(!summary.corpus_never_depletes);
    }

    #[test]
    fn test_zero_contributions_still_grows() {
        let profile = test_profile();
        let scenario = test_scenario();
        let mut scenario_mut = scenario.clone();
        scenario_mut.instruments[0].monthly_contribution_cents = 0;
        let results = projection(&profile, &scenario_mut);

        // With 7% return and no contributions, corpus still grows
        assert!(results[0].corpus_cents > 10_000_000);
    }

    #[test]
    fn test_hundred_percent_tax_reduces_withdrawals() {
        let mut profile = test_profile();
        profile.effective_tax_rate = 100.0;
        profile.retirement_age = 31;
        profile.life_expectancy = 40;
        let mut scenario = test_scenario();
        scenario.incomes.clear();
        let results = projection(&profile, &scenario);

        // With 100% tax, gross withdrawal doubles — corpus depletes faster
        let depletes = results.iter().any(|p| p.corpus_cents <= 0);
        assert!(depletes, "100% tax should cause faster depletion");
    }

    #[test]
    fn test_retirement_equals_current_age() {
        let mut profile = test_profile();
        profile.retirement_age = profile.current_age; // Retire immediately
        let scenario = test_scenario();
        let results = projection(&profile, &scenario);

        // First year should be marked as retired
        assert!(results[0].is_retired);
    }

    #[test]
    fn test_pessimistic_less_than_optimistic() {
        let profile = test_profile();
        let scenario = test_scenario();
        let base = projection(&profile, &scenario);
        let pess = projection_with_band(&profile, &scenario, ReturnBand::Pessimistic);
        let opti = projection_with_band(&profile, &scenario, ReturnBand::Optimistic);

        // At retirement, optimistic > base > pessimistic
        let retire_age = profile.retirement_age;
        let base_corpus = base.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);
        let pess_corpus = pess.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);
        let opti_corpus = opti.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);

        assert!(pess_corpus < base_corpus, "Pessimistic should be less than base");
        assert!(base_corpus < opti_corpus, "Base should be less than optimistic");
    }

    #[test]
    fn test_one_time_inflow_increases_corpus() {
        use crate::models::OneTimeEvent;
        let profile = test_profile();
        let scenario_base = test_scenario();
        let mut scenario_event = test_scenario();

        scenario_event.one_time_events.push(OneTimeEvent {
            id: uuid::Uuid::nil(),
            name: "Inheritance".to_string(),
            age: 40,
            amount_cents: 5_000_000, // $50k inflow in cents
            notes: String::new(),
        });

        let base_results = projection(&profile, &scenario_base);
        let event_results = projection(&profile, &scenario_event);

        // Corpus at retirement should be higher with the inflow
        let retire_age = profile.retirement_age;
        let base_corpus = base_results.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);
        let event_corpus = event_results.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);

        assert!(event_corpus > base_corpus, "Inflow event should increase retirement corpus");
    }

    #[test]
    fn test_one_time_outflow_decreases_corpus() {
        use crate::models::OneTimeEvent;
        let profile = test_profile();
        let scenario_base = test_scenario();
        let mut scenario_event = test_scenario();

        scenario_event.one_time_events.push(OneTimeEvent {
            id: uuid::Uuid::nil(),
            name: "Home Purchase".to_string(),
            age: 40,
            amount_cents: -5_000_000, // $50k outflow in cents
            notes: String::new(),
        });

        let base_results = projection(&profile, &scenario_base);
        let event_results = projection(&profile, &scenario_event);

        let retire_age = profile.retirement_age;
        let base_corpus = base_results.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);
        let event_corpus = event_results.iter().find(|p| p.age == retire_age).map(|p| p.corpus_cents).unwrap_or(0);

        assert!(event_corpus < base_corpus, "Outflow event should decrease retirement corpus");
    }

    #[test]
    fn test_corpus_never_negative_in_results() {
        let profile = test_profile();
        let scenario = test_scenario();
        let results = projection(&profile, &scenario);

        for p in &results {
            assert!(p.corpus_cents >= 0, "Corpus should never be negative in results, got {} at age {}", p.corpus_cents, p.age);
        }
    }

    #[test]
    fn test_depletion_age_not_before_retirement() {
        let mut profile = test_profile();
        profile.retirement_age = 31;
        profile.life_expectancy = 40;
        let mut scenario = test_scenario();
        scenario.instruments[0].current_value_cents = 1_000_000;
        scenario.instruments[0].monthly_contribution_cents = 0;
        scenario.expenses[0].monthly_amount_cents = 500_000;
        scenario.incomes.clear();

        let results = projection(&profile, &scenario);
        let summary = summarize_projection(&profile, &results);

        if let Some(depletion_age) = summary.depletion_age {
            assert!(
                depletion_age >= profile.retirement_age,
                "Depletion age {} should be >= retirement age {}",
                depletion_age,
                profile.retirement_age
            );
        }
    }
}
