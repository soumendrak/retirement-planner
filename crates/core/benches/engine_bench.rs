use criterion::{black_box, criterion_group, criterion_main, Criterion};

use retirement_planner_core::currency::Currency;
use retirement_planner_core::engine::{projection, projection_with_band, ReturnBand};
use retirement_planner_core::models::{
    AssetClass, Expense, ExpenseCategory, Income, InflationConfig, Instrument, Profile, Scenario,
};

fn make_profile() -> Profile {
    Profile {
        id: uuid::Uuid::nil(),
        name: "Bench".to_string(),
        current_age: 30,
        retirement_age: 60,
        life_expectancy: 85,
        base_currency: Currency::USD,
        exchange_rate: 83.0,
        effective_tax_rate: 25.0,
    }
}

fn make_scenario(n_instruments: usize) -> Scenario {
    let instruments = (0..n_instruments)
        .map(|i| Instrument {
            id: uuid::Uuid::new_v4(),
            name: format!("Fund {}", i),
            asset_class: AssetClass::Equity,
            expected_annual_return: 7.0,
            current_value_cents: 10_000_000,
            monthly_contribution_cents: 100_000,
        })
        .collect();

    Scenario {
        id: uuid::Uuid::nil(),
        name: "Bench".to_string(),
        profile_id: uuid::Uuid::nil(),
        instruments,
        expenses: vec![Expense {
            id: uuid::Uuid::nil(),
            category: ExpenseCategory::Housing,
            monthly_amount_cents: 200_000,
            inflation_rate: Some(3.0),
        }],
        incomes: vec![Income {
            id: uuid::Uuid::nil(),
            name: "Salary".to_string(),
            monthly_amount_cents: 500_000,
            start_year: 0,
            end_year: Some(29),
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

fn bench_projection_single(c: &mut Criterion) {
    let profile = make_profile();
    let scenario = make_scenario(1);
    c.bench_function("projection_single_instrument", |b| {
        b.iter(|| projection(black_box(&profile), black_box(&scenario)));
    });
}

fn bench_projection_ten(c: &mut Criterion) {
    let profile = make_profile();
    let scenario = make_scenario(10);
    c.bench_function("projection_10_instruments", |b| {
        b.iter(|| projection(black_box(&profile), black_box(&scenario)));
    });
}

fn bench_projection_bands(c: &mut Criterion) {
    let profile = make_profile();
    let scenario = make_scenario(5);
    c.bench_function("projection_all_3_bands", |b| {
        b.iter(|| {
            let _ = projection(black_box(&profile), black_box(&scenario));
            let _ = projection_with_band(black_box(&profile), black_box(&scenario), ReturnBand::Pessimistic);
            let _ = projection_with_band(black_box(&profile), black_box(&scenario), ReturnBand::Optimistic);
        });
    });
}

criterion_group!(
    benches,
    bench_projection_single,
    bench_projection_ten,
    bench_projection_bands
);
criterion_main!(benches);
