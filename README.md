# Retirement Planner

A privacy-first, offline-only retirement planning desktop app built in Rust. Supports USD and INR currencies with multi-scenario comparison dashboards and rich visualizations.

## Features

- **Fully offline** — no internet access required, no cloud dependencies
- **Privacy-first** — all data stored locally, user-controlled
- **Multi-currency** — USD (comma formatting) and INR (lakh/crore formatting)
- **Profile setup** — age, retirement age, life expectancy, flat tax rate
- **Investment instruments** — full CRUD with asset classes (Equity, Debt, Real Estate, Cash, Other)
- **Monthly expenses** — categorized with per-category or blanket inflation rates
- **Income sources** — salary, pension, rental with start/end years and annual growth
- **Multiple scenarios** — create, clone, and compare what-if retirement plans
- **Rich dashboard** — side-by-side scenario comparison with:
  - Corpus over time line chart (multi-scenario overlay)
  - Expenses vs withdrawals bar chart
  - Asset allocation pie chart
  - Year-by-year projection table
- **JSON export/import** — save and load plans as portable JSON files
- **Input validation** — validates imported data for correctness

## Screenshots

The app features a dark sidebar with navigation and a light content area:

```
┌──────────┬──────────────────────────────────────────┐
│ Sidebar  │  Content Area                            │
│          │                                          │
│ Dashboard│  Profile / Instruments / Expenses /       │
│ Profile  │  Income / Scenarios / Dashboard          │
│ Instru.  │                                          │
│ Expenses │  [Charts & Tables]                       │
│ Income   │                                          │
│ Scenarios│                                          │
│          │                                          │
│ [Run]    │                                          │
│ [Ex][Im] │                                          │
└──────────┴──────────────────────────────────────────┘
```

## Tech Stack

| Layer | Choice |
|---|---|
| Language | Rust |
| GUI | iced 0.13 (pure Rust, wgpu-based) |
| Storage | rusqlite (bundled SQLite) |
| Charts | iced canvas widget (pure Rust) |
| Serialization | serde + serde_json |
| IDs | uuid v4 |

## Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **macOS / Windows / Linux** — native desktop app

No other dependencies required. SQLite is bundled.

## Build & Run

```bash
# Clone the repository
git clone https://github.com/soumendrak/retirement-planner.git
cd retirement-planner

# Build (first build takes a few minutes for iced/wgpu)
cargo build --release

# Run
cargo run -p retirement-planner-ui --release
```

For development:

```bash
# Debug build (faster compile, slower runtime)
cargo run -p retirement-planner-ui

# Run tests
cargo test --workspace
```

## Project Structure

```
retirement-planner/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── core/                   # domain logic — zero GUI deps
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # Profile, Instrument, Expense, Income, Scenario
│   │       ├── engine.rs       # year-by-year projection engine
│   │       ├── currency.rs     # USD/INR formatting, cents/paise storage
│   │       ├── export.rs       # JSON export/import
│   │       └── storage.rs      # SQLite CRUD operations
│   └── ui/                     # iced GUI shell
│       └── src/
│           ├── main.rs
│           ├── app.rs          # top-level app with sidebar navigation
│           ├── theme.rs        # color and spacing constants
│           ├── views/          # Profile, Instruments, Expenses, Income, Scenarios, Dashboard
│           └── charts/         # Canvas-based charts (corpus line, withdrawal bar, allocation pie)
├── AGENTS.md                   # living project context document
└── README.md
```

## How to Use

1. **Profile** — Enter your name, current age, retirement age, life expectancy, currency, and tax rate
2. **Instruments** — Add your investments (stocks, bonds, mutual funds) with expected returns and monthly contributions
3. **Expenses** — Add monthly expenses by category with optional per-category inflation rates
4. **Income** — Add income sources (salary, pension, rental) with start/end years
5. **Scenarios** — Create multiple what-if scenarios (e.g., "Conservative", "Aggressive")
6. **Run Projections** — Click the green button in the sidebar to run the simulation
7. **Dashboard** — View comparison cards, charts, and year-by-year projections

## JSON Export/Import

- **Export**: Click "Export JSON" — saves to `~/Documents/retirement_plan.json`
- **Import**: Click "Import JSON" — loads from `~/Documents/retirement_plan.json`

The JSON format is versioned for forward compatibility.

## Key Design Decisions

- **All monetary values stored as `i64` cents/paise** — avoids floating-point precision errors
- **Per-category inflation rates** — healthcare inflates differently than food
- **Simple tax model** — flat effective tax rate (no complex tax bracket modeling)
- **No internet** — fully functional offline, no API calls
- **Pure Rust charts** — iced canvas widget, no C++ dependencies

## Testing

```bash
# Run all tests
cargo test --workspace

# Run core tests only
cargo test -p retirement-planner-core
```

12 unit tests cover:
- Currency formatting (USD commas, INR lakh/crore)
- Projection engine (corpus growth, depletion detection, SWR calculation)
- SQLite CRUD operations (profile and scenario round-trips)

## License

MIT
