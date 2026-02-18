//! SQLite storage layer for persistence.
//!
//! Uses a single database file stored in the platform's app-data directory.

use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

use crate::models::{
    Expense, ExpenseCategory, Income, InflationConfig, Instrument, Profile, Scenario,
};

/// Storage error type.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Main storage handle wrapping a SQLite connection.
pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    /// Create or open the database at the given path.
    pub fn new(path: &PathBuf) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.initialize_schema()?;
        Ok(storage)
    }

    /// Create or open the database in the platform's app-data directory.
    pub fn in_app_dir() -> Result<Self, StorageError> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No data dir"))?;
        let app_dir = data_dir.join("retirement-planner");
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("data.db");
        Self::new(&db_path)
    }

    /// Initialize the database schema.
    fn initialize_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            -- Schema version tracking
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO schema_version (version)
                SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM schema_version);

            -- Profiles table
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                current_age INTEGER NOT NULL,
                retirement_age INTEGER NOT NULL,
                life_expectancy INTEGER NOT NULL,
                base_currency TEXT NOT NULL,
                exchange_rate REAL NOT NULL,
                effective_tax_rate REAL NOT NULL
            );

            -- Scenarios table
            CREATE TABLE IF NOT EXISTS scenarios (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                profile_id TEXT NOT NULL,
                instruments_json TEXT NOT NULL DEFAULT '[]',
                expenses_json TEXT NOT NULL DEFAULT '[]',
                incomes_json TEXT NOT NULL DEFAULT '[]',
                one_time_events_json TEXT NOT NULL DEFAULT '[]',
                inflation_blanket_rate REAL NOT NULL DEFAULT 3.0,
                inflation_use_blanket INTEGER NOT NULL DEFAULT 0,
                notes TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            );

            -- Instruments table (for easier querying)
            CREATE TABLE IF NOT EXISTS instruments (
                id TEXT PRIMARY KEY,
                scenario_id TEXT NOT NULL,
                name TEXT NOT NULL,
                asset_class TEXT NOT NULL,
                expected_annual_return REAL NOT NULL,
                current_value_cents INTEGER NOT NULL,
                monthly_contribution_cents INTEGER NOT NULL,
                FOREIGN KEY (scenario_id) REFERENCES scenarios(id) ON DELETE CASCADE
            );

            -- Expenses table
            CREATE TABLE IF NOT EXISTS expenses (
                id TEXT PRIMARY KEY,
                scenario_id TEXT NOT NULL,
                category TEXT NOT NULL,
                monthly_amount_cents INTEGER NOT NULL,
                inflation_rate REAL
            );

            -- Incomes table
            CREATE TABLE IF NOT EXISTS incomes (
                id TEXT PRIMARY KEY,
                scenario_id TEXT NOT NULL,
                name TEXT NOT NULL,
                monthly_amount_cents INTEGER NOT NULL,
                start_year INTEGER NOT NULL DEFAULT 0,
                end_year INTEGER,
                annual_growth REAL NOT NULL DEFAULT 0
            );

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_scenarios_profile ON scenarios(profile_id);
            CREATE INDEX IF NOT EXISTS idx_instruments_scenario ON instruments(scenario_id);
            CREATE INDEX IF NOT EXISTS idx_expenses_scenario ON expenses(scenario_id);
            CREATE INDEX IF NOT EXISTS idx_incomes_scenario ON incomes(scenario_id);
            "#,
        )?;

        Ok(())
    }

    // ========== Profile CRUD ==========

    /// Create a new profile.
    pub fn create_profile(&self, profile: &Profile) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO profiles (id, name, current_age, retirement_age, life_expectancy, 
                                  base_currency, exchange_rate, effective_tax_rate)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                profile.id.to_string(),
                profile.name,
                profile.current_age,
                profile.retirement_age,
                profile.life_expectancy,
                profile.base_currency.code(),
                profile.exchange_rate,
                profile.effective_tax_rate,
            ],
        )?;
        Ok(())
    }

    /// Get a profile by ID.
    pub fn get_profile(&self, id: Uuid) -> Result<Option<Profile>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, current_age, retirement_age, life_expectancy, 
                   base_currency, exchange_rate, effective_tax_rate
            FROM profiles WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row(params![id.to_string()], |row| {
            Ok(Profile {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                current_age: row.get(2)?,
                retirement_age: row.get(3)?,
                life_expectancy: row.get(4)?,
                base_currency: row.get::<_, String>(5)?.parse().unwrap(),
                exchange_rate: row.get(6)?,
                effective_tax_rate: row.get(7)?,
            })
        });

        match result {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all profiles.
    pub fn get_all_profiles(&self) -> Result<Vec<Profile>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, current_age, retirement_age, life_expectancy, 
                   base_currency, exchange_rate, effective_tax_rate
            FROM profiles ORDER BY name
            "#,
        )?;

        let profiles = stmt
            .query_map([], |row| {
                Ok(Profile {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    name: row.get(1)?,
                    current_age: row.get(2)?,
                    retirement_age: row.get(3)?,
                    life_expectancy: row.get(4)?,
                    base_currency: row.get::<_, String>(5)?.parse().unwrap(),
                    exchange_rate: row.get(6)?,
                    effective_tax_rate: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    /// Update a profile.
    pub fn update_profile(&self, profile: &Profile) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            r#"
            UPDATE profiles SET 
                name = ?2,
                current_age = ?3,
                retirement_age = ?4,
                life_expectancy = ?5,
                base_currency = ?6,
                exchange_rate = ?7,
                effective_tax_rate = ?8
            WHERE id = ?1
            "#,
            params![
                profile.id.to_string(),
                profile.name,
                profile.current_age,
                profile.retirement_age,
                profile.life_expectancy,
                profile.base_currency.code(),
                profile.exchange_rate,
                profile.effective_tax_rate,
            ],
        )?;

        if rows == 0 {
            return Err(StorageError::NotFound(format!("Profile {}", profile.id)));
        }

        Ok(())
    }

    /// Delete a profile and all its scenarios.
    pub fn delete_profile(&self, id: Uuid) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM profiles WHERE id = ?1", params![id.to_string()])?;

        if rows == 0 {
            return Err(StorageError::NotFound(format!("Profile {}", id)));
        }

        Ok(())
    }

    // ========== Scenario CRUD ==========

    /// Create a new scenario with its instruments, expenses, and incomes.
    pub fn create_scenario(&self, scenario: &Scenario) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let one_time_events_json = serde_json::to_string(&scenario.one_time_events)?;

        // Insert scenario
        tx.execute(
            r#"
            INSERT INTO scenarios (id, name, profile_id, inflation_blanket_rate, 
                                   inflation_use_blanket, notes, one_time_events_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                scenario.id.to_string(),
                scenario.name,
                scenario.profile_id.to_string(),
                scenario.inflation_config.blanket_rate,
                scenario.inflation_config.use_blanket as i32,
                scenario.notes,
                one_time_events_json,
            ],
        )?;

        // Insert instruments
        for inst in &scenario.instruments {
            tx.execute(
                r#"
                INSERT INTO instruments (id, scenario_id, name, asset_class, 
                                        expected_annual_return, current_value_cents, 
                                        monthly_contribution_cents)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    inst.id.to_string(),
                    scenario.id.to_string(),
                    inst.name,
                    inst.asset_class.as_str(),
                    inst.expected_annual_return,
                    inst.current_value_cents,
                    inst.monthly_contribution_cents,
                ],
            )?;
        }

        // Insert expenses
        for exp in &scenario.expenses {
            tx.execute(
                r#"
                INSERT INTO expenses (id, scenario_id, category, monthly_amount_cents, 
                                      inflation_rate)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    exp.id.to_string(),
                    scenario.id.to_string(),
                    exp.category.as_str(),
                    exp.monthly_amount_cents,
                    exp.inflation_rate,
                ],
            )?;
        }

        // Insert incomes
        for inc in &scenario.incomes {
            tx.execute(
                r#"
                INSERT INTO incomes (id, scenario_id, name, monthly_amount_cents, 
                                     start_year, end_year, annual_growth)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    inc.id.to_string(),
                    scenario.id.to_string(),
                    inc.name,
                    inc.monthly_amount_cents,
                    inc.start_year,
                    inc.end_year,
                    inc.annual_growth,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get a scenario by ID with all its related data.
    pub fn get_scenario(&self, id: Uuid) -> Result<Option<Scenario>, StorageError> {
        let conn = self.conn.lock().unwrap();

        // Get scenario
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, profile_id, inflation_blanket_rate, inflation_use_blanket, notes, one_time_events_json
            FROM scenarios WHERE id = ?1
            "#,
        )?;

        let scenario_result = stmt.query_row(params![id.to_string()], |row| {
            let events_json: String = row.get(6).unwrap_or_else(|_| "[]".to_string());
            Ok(Scenario {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                profile_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap(),
                inflation_config: InflationConfig {
                    blanket_rate: row.get(3)?,
                    use_blanket: row.get::<_, i32>(4)? != 0,
                },
                notes: row.get(5)?,
                instruments: Vec::new(),
                expenses: Vec::new(),
                incomes: Vec::new(),
                one_time_events: serde_json::from_str(&events_json).unwrap_or_default(),
            })
        });

        let mut scenario = match scenario_result {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Get instruments
        let mut inst_stmt = conn.prepare(
            r#"
            SELECT id, name, asset_class, expected_annual_return, 
                   current_value_cents, monthly_contribution_cents
            FROM instruments WHERE scenario_id = ?1
            "#,
        )?;

        scenario.instruments = inst_stmt
            .query_map(params![id.to_string()], |row| {
                Ok(Instrument {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    name: row.get(1)?,
                    asset_class: row.get::<_, String>(2)?.parse().unwrap(),
                    expected_annual_return: row.get(3)?,
                    current_value_cents: row.get(4)?,
                    monthly_contribution_cents: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Get expenses
        let mut exp_stmt = conn.prepare(
            r#"
            SELECT id, category, monthly_amount_cents, inflation_rate
            FROM expenses WHERE scenario_id = ?1
            "#,
        )?;

        scenario.expenses = exp_stmt
            .query_map(params![id.to_string()], |row| {
                let category_str = row.get::<_, String>(1)?;
                let category = parse_expense_category(&category_str);
                Ok(Expense {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    category,
                    monthly_amount_cents: row.get(2)?,
                    inflation_rate: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Get incomes
        let mut inc_stmt = conn.prepare(
            r#"
            SELECT id, name, monthly_amount_cents, start_year, end_year, annual_growth
            FROM incomes WHERE scenario_id = ?1
            "#,
        )?;

        scenario.incomes = inc_stmt
            .query_map(params![id.to_string()], |row| {
                Ok(Income {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    name: row.get(1)?,
                    monthly_amount_cents: row.get(2)?,
                    start_year: row.get(3)?,
                    end_year: row.get(4)?,
                    annual_growth: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(scenario))
    }

    /// Get all scenarios for a profile.
    pub fn get_scenarios_for_profile(&self, profile_id: Uuid) -> Result<Vec<Scenario>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id FROM scenarios WHERE profile_id = ?1 ORDER BY name",
        )?;

        let ids: Vec<Uuid> = stmt
            .query_map(params![profile_id.to_string()], |row| {
                Ok(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap())
            })?
            .collect::<Result<Vec<_>, _>>()?;

        drop(stmt);
        drop(conn);

        let mut scenarios = Vec::new();
        for id in ids {
            if let Some(scenario) = self.get_scenario(id)? {
                scenarios.push(scenario);
            }
        }

        Ok(scenarios)
    }

    /// Update a scenario.
    pub fn update_scenario(&self, scenario: &Scenario) -> Result<(), StorageError> {
        // For simplicity, delete and recreate
        self.delete_scenario(scenario.id)?;
        self.create_scenario(scenario)
    }

    /// Delete a scenario and all its child records.
    pub fn delete_scenario(&self, id: Uuid) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Delete child rows first
        conn.execute("DELETE FROM instruments WHERE scenario_id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM expenses WHERE scenario_id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM incomes WHERE scenario_id = ?1", params![id.to_string()])?;

        let rows = conn.execute("DELETE FROM scenarios WHERE id = ?1", params![id.to_string()])?;

        if rows == 0 {
            return Err(StorageError::NotFound(format!("Scenario {}", id)));
        }

        Ok(())
    }
}

/// Parse an expense category string.
fn parse_expense_category(s: &str) -> ExpenseCategory {
    match s {
        "Housing" => ExpenseCategory::Housing,
        "Healthcare" => ExpenseCategory::Healthcare,
        "Food" => ExpenseCategory::Food,
        "Transport" => ExpenseCategory::Transport,
        "Leisure" => ExpenseCategory::Leisure,
        other => ExpenseCategory::Other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AssetClass;
    use tempfile::tempdir;

    fn test_storage() -> (Storage, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let storage = Storage::new(&path).unwrap();
        (storage, dir)
    }

    #[test]
    fn test_profile_crud() {
        let (storage, _dir) = test_storage();

        let profile = Profile::new("Test User".to_string());
        storage.create_profile(&profile).unwrap();

        let retrieved = storage.get_profile(profile.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test User");

        let all = storage.get_all_profiles().unwrap();
        assert_eq!(all.len(), 1);

        let mut updated = retrieved.clone();
        updated.name = "Updated Name".to_string();
        storage.update_profile(&updated).unwrap();

        let after_update = storage.get_profile(profile.id).unwrap().unwrap();
        assert_eq!(after_update.name, "Updated Name");

        storage.delete_profile(profile.id).unwrap();
        assert!(storage.get_profile(profile.id).unwrap().is_none());
    }

    #[test]
    fn test_scenario_crud() {
        let (storage, _dir) = test_storage();

        let profile = Profile::new("Test".to_string());
        storage.create_profile(&profile).unwrap();

        let mut scenario = Scenario::new("Conservative".to_string(), profile.id);
        scenario.instruments.push(Instrument::new(
            "US Equity".to_string(),
            AssetClass::Equity,
        ));
        scenario.expenses.push(Expense::new(
            ExpenseCategory::Housing,
            100000,
        ));
        scenario.incomes.push(Income::new("Salary".to_string(), 500000));

        storage.create_scenario(&scenario).unwrap();

        let retrieved = storage.get_scenario(scenario.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Conservative");
        assert_eq!(retrieved.instruments.len(), 1);
        assert_eq!(retrieved.expenses.len(), 1);
        assert_eq!(retrieved.incomes.len(), 1);

        let scenarios = storage.get_scenarios_for_profile(profile.id).unwrap();
        assert_eq!(scenarios.len(), 1);

        storage.delete_scenario(scenario.id).unwrap();
        assert!(storage.get_scenario(scenario.id).unwrap().is_none());
    }
}
