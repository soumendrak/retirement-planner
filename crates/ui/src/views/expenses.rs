use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Element, Length};

use retirement_planner_core::currency::{format_currency, Currency};
use retirement_planner_core::models::{Expense, ExpenseCategory, InflationConfig};

#[derive(Debug, Clone)]
pub enum ExpensesMessage {
    CategorySelected(CategoryOption),
    CustomCategoryChanged(String),
    AmountChanged(String),
    InflationRateChanged(String),
    UseCustomInflation(bool),
    BlanketRateChanged(String),
    UseBlanketToggle(bool),
    Add,
    Edit(usize),
    Delete(usize),
    CancelEdit,
    SaveEdit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryOption {
    Housing,
    Healthcare,
    Food,
    Transport,
    Leisure,
    Custom,
}

impl std::fmt::Display for CategoryOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CategoryOption::Housing => write!(f, "Housing"),
            CategoryOption::Healthcare => write!(f, "Healthcare"),
            CategoryOption::Food => write!(f, "Food"),
            CategoryOption::Transport => write!(f, "Transport"),
            CategoryOption::Leisure => write!(f, "Leisure"),
            CategoryOption::Custom => write!(f, "Other (custom)"),
        }
    }
}

const CATEGORY_OPTIONS: &[CategoryOption] = &[
    CategoryOption::Housing,
    CategoryOption::Healthcare,
    CategoryOption::Food,
    CategoryOption::Transport,
    CategoryOption::Leisure,
    CategoryOption::Custom,
];

#[derive(Debug, Clone)]
pub struct ExpensesView {
    pub expenses: Vec<Expense>,
    pub inflation_config: InflationConfig,
    pub selected_category: CategoryOption,
    pub custom_category: String,
    pub amount: String,
    pub inflation_rate: String,
    pub use_custom_inflation: bool,
    pub blanket_rate: String,
    pub editing_index: Option<usize>,
    pub currency: Currency,
}

impl Default for ExpensesView {
    fn default() -> Self {
        Self {
            expenses: Vec::new(),
            inflation_config: InflationConfig::default(),
            selected_category: CategoryOption::Housing,
            custom_category: String::new(),
            amount: String::new(),
            inflation_rate: "3.0".to_string(),
            use_custom_inflation: false,
            blanket_rate: "3.0".to_string(),
            editing_index: None,
            currency: Currency::USD,
        }
    }
}

impl ExpensesView {
    fn clear_form(&mut self) {
        self.selected_category = CategoryOption::Housing;
        self.custom_category.clear();
        self.amount.clear();
        self.inflation_rate = "3.0".to_string();
        self.use_custom_inflation = false;
        self.editing_index = None;
    }

    fn category_from_option(&self) -> ExpenseCategory {
        match &self.selected_category {
            CategoryOption::Housing => ExpenseCategory::Housing,
            CategoryOption::Healthcare => ExpenseCategory::Healthcare,
            CategoryOption::Food => ExpenseCategory::Food,
            CategoryOption::Transport => ExpenseCategory::Transport,
            CategoryOption::Leisure => ExpenseCategory::Leisure,
            CategoryOption::Custom => ExpenseCategory::Other(self.custom_category.clone()),
        }
    }

    pub fn update(&mut self, message: ExpensesMessage) {
        match message {
            ExpensesMessage::CategorySelected(cat) => self.selected_category = cat,
            ExpensesMessage::CustomCategoryChanged(s) => self.custom_category = s,
            ExpensesMessage::AmountChanged(a) => self.amount = a,
            ExpensesMessage::InflationRateChanged(r) => self.inflation_rate = r,
            ExpensesMessage::UseCustomInflation(b) => self.use_custom_inflation = b,
            ExpensesMessage::BlanketRateChanged(r) => {
                self.blanket_rate = r.clone();
                if let Ok(rate) = r.parse::<f64>() {
                    self.inflation_config.blanket_rate = rate;
                }
            }
            ExpensesMessage::UseBlanketToggle(b) => {
                self.inflation_config.use_blanket = b;
            }
            ExpensesMessage::Add => {
                if !self.amount.is_empty() {
                    let cents = (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                    let inflation = if self.use_custom_inflation {
                        Some(self.inflation_rate.parse().unwrap_or(3.0))
                    } else {
                        None
                    };
                    let expense = Expense {
                        id: uuid::Uuid::new_v4(),
                        category: self.category_from_option(),
                        monthly_amount_cents: cents,
                        inflation_rate: inflation,
                    };
                    self.expenses.push(expense);
                    self.clear_form();
                }
            }
            ExpensesMessage::Edit(idx) => {
                if let Some(exp) = self.expenses.get(idx) {
                    self.selected_category = match &exp.category {
                        ExpenseCategory::Housing => CategoryOption::Housing,
                        ExpenseCategory::Healthcare => CategoryOption::Healthcare,
                        ExpenseCategory::Food => CategoryOption::Food,
                        ExpenseCategory::Transport => CategoryOption::Transport,
                        ExpenseCategory::Leisure => CategoryOption::Leisure,
                        ExpenseCategory::Other(s) => {
                            self.custom_category = s.clone();
                            CategoryOption::Custom
                        }
                    };
                    self.amount = format!("{:.0}", exp.monthly_amount_cents as f64 / 100.0);
                    self.use_custom_inflation = exp.inflation_rate.is_some();
                    self.inflation_rate = format!(
                        "{:.1}",
                        exp.inflation_rate.unwrap_or(self.inflation_config.blanket_rate)
                    );
                    self.editing_index = Some(idx);
                }
            }
            ExpensesMessage::Delete(idx) => {
                if idx < self.expenses.len() {
                    self.expenses.remove(idx);
                }
            }
            ExpensesMessage::CancelEdit => self.clear_form(),
            ExpensesMessage::SaveEdit => {
                if let Some(idx) = self.editing_index {
                    let category = self.category_from_option();
                    let cents = (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                    let inflation = if self.use_custom_inflation {
                        Some(self.inflation_rate.parse().unwrap_or(3.0))
                    } else {
                        None
                    };
                    if let Some(exp) = self.expenses.get_mut(idx) {
                        exp.category = category;
                        exp.monthly_amount_cents = cents;
                        exp.inflation_rate = inflation;
                    }
                }
                self.clear_form();
            }
        }
    }

    pub fn view(&self) -> Element<'_, ExpensesMessage> {
        let title = text("Monthly Expenses").size(32);

        // Inflation config
        let blanket_section = column![
            text("Inflation Settings").size(18),
            row![
                checkbox("Use single blanket rate for all categories", self.inflation_config.use_blanket)
                    .on_toggle(ExpensesMessage::UseBlanketToggle),
            ],
            row![
                text("Blanket inflation rate (%): ").size(14),
                text_input("3.0", &self.blanket_rate)
                    .on_input(ExpensesMessage::BlanketRateChanged)
                    .width(80)
                    .size(14),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(8);

        // Add/edit form
        let cat_picker = column![
            text("Category").size(14),
            pick_list(
                CATEGORY_OPTIONS,
                Some(self.selected_category.clone()),
                ExpensesMessage::CategorySelected,
            )
            .width(Length::Fill),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let custom_field = if self.selected_category == CategoryOption::Custom {
            column![
                text("Custom Category").size(14),
                text_input("e.g. Education", &self.custom_category)
                    .on_input(ExpensesMessage::CustomCategoryChanged)
                    .size(16),
            ]
            .spacing(4)
            .width(Length::FillPortion(2))
        } else {
            column![].width(Length::FillPortion(2))
        };

        let amount_field = column![
            text("Monthly Amount").size(14),
            text_input("2000", &self.amount)
                .on_input(ExpensesMessage::AmountChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let inflation_field = column![
            checkbox("Custom inflation rate", self.use_custom_inflation)
                .on_toggle(ExpensesMessage::UseCustomInflation),
            text_input("3.0", &self.inflation_rate)
                .on_input(ExpensesMessage::InflationRateChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let form_buttons = if self.editing_index.is_some() {
            row![
                button(text("Save").size(14)).padding(8).on_press(ExpensesMessage::SaveEdit),
                button(text("Cancel").size(14)).padding(8).on_press(ExpensesMessage::CancelEdit),
            ]
            .spacing(8)
        } else {
            row![
                button(text("Add Expense").size(14)).padding(8).on_press(ExpensesMessage::Add),
            ]
        };

        let form = column![
            row![cat_picker, custom_field, amount_field, inflation_field].spacing(12),
            form_buttons,
        ]
        .spacing(12);

        // Expense list
        let header = row![
            text("Category").size(14).width(Length::FillPortion(2)),
            text("Monthly").size(14).width(Length::FillPortion(2)),
            text("Inflation").size(14).width(Length::FillPortion(2)),
            text("Actions").size(14).width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([8u16, 0u16]);

        let mut list = column![].spacing(4);
        for (idx, exp) in self.expenses.iter().enumerate() {
            let inflation_text = match exp.inflation_rate {
                Some(r) => format!("{:.1}% (custom)", r),
                None => format!("{:.1}% (blanket)", self.inflation_config.blanket_rate),
            };

            let item = row![
                text(exp.category.as_str()).size(14).width(Length::FillPortion(2)),
                text(format_currency(exp.monthly_amount_cents, self.currency))
                    .size(14)
                    .width(Length::FillPortion(2)),
                text(inflation_text).size(14).width(Length::FillPortion(2)),
                row![
                    button(text("Edit").size(12)).padding(4).on_press(ExpensesMessage::Edit(idx)),
                    button(text("Del").size(12)).padding(4).on_press(ExpensesMessage::Delete(idx)),
                ]
                .spacing(4)
                .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .padding([6u16, 0u16]);

            list = list.push(item);
        }

        // Total
        let total_cents: i64 = self.expenses.iter().map(|e| e.monthly_amount_cents).sum();
        let total_row = row![
            text("Total Monthly Expenses:").size(16),
            text(format_currency(total_cents, self.currency)).size(16),
        ]
        .spacing(8);

        let content = column![
            title,
            blanket_section,
            text("Add Expense").size(18),
            form,
            header,
            scrollable(list),
            total_row,
        ]
        .spacing(16)
        .max_width(900);

        container(content).padding(24).width(Length::Fill).into()
    }
}
