use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use retirement_planner_core::currency::{format_currency, Currency};
use retirement_planner_core::models::Income;

#[derive(Debug, Clone)]
pub enum IncomeMessage {
    NameChanged(String),
    AmountChanged(String),
    StartYearChanged(String),
    EndYearChanged(String),
    GrowthChanged(String),
    HasEndYear(bool),
    Add,
    Edit(usize),
    Delete(usize),
    CancelEdit,
    SaveEdit,
}

#[derive(Debug, Clone)]
pub struct IncomeView {
    pub incomes: Vec<Income>,
    pub name: String,
    pub amount: String,
    pub start_year: String,
    pub end_year: String,
    pub has_end_year: bool,
    pub annual_growth: String,
    pub editing_index: Option<usize>,
    pub currency: Currency,
}

impl Default for IncomeView {
    fn default() -> Self {
        Self {
            incomes: Vec::new(),
            name: String::new(),
            amount: String::new(),
            start_year: "0".to_string(),
            end_year: String::new(),
            has_end_year: false,
            annual_growth: "3.0".to_string(),
            editing_index: None,
            currency: Currency::USD,
        }
    }
}

impl IncomeView {
    fn clear_form(&mut self) {
        self.name.clear();
        self.amount.clear();
        self.start_year = "0".to_string();
        self.end_year.clear();
        self.has_end_year = false;
        self.annual_growth = "3.0".to_string();
        self.editing_index = None;
    }

    pub fn update(&mut self, message: IncomeMessage) {
        match message {
            IncomeMessage::NameChanged(n) => self.name = n,
            IncomeMessage::AmountChanged(a) => self.amount = a,
            IncomeMessage::StartYearChanged(y) => self.start_year = y,
            IncomeMessage::EndYearChanged(y) => self.end_year = y,
            IncomeMessage::GrowthChanged(g) => self.annual_growth = g,
            IncomeMessage::HasEndYear(b) => self.has_end_year = b,
            IncomeMessage::Add => {
                if !self.name.is_empty() && !self.amount.is_empty() {
                    let cents = (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                    let income = Income {
                        id: uuid::Uuid::new_v4(),
                        name: self.name.clone(),
                        monthly_amount_cents: cents,
                        start_year: self.start_year.parse().unwrap_or(0),
                        end_year: if self.has_end_year {
                            self.end_year.parse().ok()
                        } else {
                            None
                        },
                        annual_growth: self.annual_growth.parse().unwrap_or(0.0),
                    };
                    self.incomes.push(income);
                    self.clear_form();
                }
            }
            IncomeMessage::Edit(idx) => {
                if let Some(inc) = self.incomes.get(idx) {
                    self.name = inc.name.clone();
                    self.amount = format!("{:.0}", inc.monthly_amount_cents as f64 / 100.0);
                    self.start_year = inc.start_year.to_string();
                    self.has_end_year = inc.end_year.is_some();
                    self.end_year = inc.end_year.map(|y| y.to_string()).unwrap_or_default();
                    self.annual_growth = format!("{:.1}", inc.annual_growth);
                    self.editing_index = Some(idx);
                }
            }
            IncomeMessage::Delete(idx) => {
                if idx < self.incomes.len() {
                    self.incomes.remove(idx);
                }
            }
            IncomeMessage::CancelEdit => self.clear_form(),
            IncomeMessage::SaveEdit => {
                if let Some(idx) = self.editing_index {
                    if let Some(inc) = self.incomes.get_mut(idx) {
                        inc.name = self.name.clone();
                        inc.monthly_amount_cents =
                            (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                        inc.start_year = self.start_year.parse().unwrap_or(0);
                        inc.end_year = if self.has_end_year {
                            self.end_year.parse().ok()
                        } else {
                            None
                        };
                        inc.annual_growth = self.annual_growth.parse().unwrap_or(0.0);
                    }
                }
                self.clear_form();
            }
        }
    }

    pub fn view(&self) -> Element<'_, IncomeMessage> {
        let title = text("Income Sources").size(32);

        let name_field = column![
            text("Source Name").size(14),
            text_input("e.g. Salary, Pension, Rental", &self.name)
                .on_input(IncomeMessage::NameChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(3));

        let amount_field = column![
            text("Monthly Amount").size(14),
            text_input("5000", &self.amount)
                .on_input(IncomeMessage::AmountChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let growth_field = column![
            text("Annual Growth (%)").size(14),
            text_input("3.0", &self.annual_growth)
                .on_input(IncomeMessage::GrowthChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let start_field = column![
            text("Start Year (0=now)").size(14),
            text_input("0", &self.start_year)
                .on_input(IncomeMessage::StartYearChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(1));

        let end_widget: Element<'_, IncomeMessage> = if self.has_end_year {
            text_input("30", &self.end_year)
                .on_input(IncomeMessage::EndYearChanged)
                .size(16)
                .into()
        } else {
            text("Indefinite").size(14).into()
        };

        let end_section = column![
            iced::widget::checkbox("Has end year", self.has_end_year)
                .on_toggle(IncomeMessage::HasEndYear),
            end_widget,
        ]
        .spacing(4)
        .width(Length::FillPortion(1));

        let form_buttons = if self.editing_index.is_some() {
            row![
                button(text("Save").size(14)).padding(8).on_press(IncomeMessage::SaveEdit),
                button(text("Cancel").size(14)).padding(8).on_press(IncomeMessage::CancelEdit),
            ]
            .spacing(8)
        } else {
            row![
                button(text("Add Income").size(14)).padding(8).on_press(IncomeMessage::Add),
            ]
        };

        let form = column![
            row![name_field, amount_field, growth_field].spacing(12),
            row![start_field, end_section].spacing(12),
            form_buttons,
        ]
        .spacing(12);

        // List
        let header = row![
            text("Source").size(14).width(Length::FillPortion(3)),
            text("Monthly").size(14).width(Length::FillPortion(2)),
            text("Growth").size(14).width(Length::FillPortion(1)),
            text("Period").size(14).width(Length::FillPortion(2)),
            text("Actions").size(14).width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([8u16, 0u16]);

        let mut list = column![].spacing(4);
        for (idx, inc) in self.incomes.iter().enumerate() {
            let period = match inc.end_year {
                Some(end) => format!("Year {} - {}", inc.start_year, end),
                None => format!("Year {} onwards", inc.start_year),
            };

            let item = row![
                text(&inc.name).size(14).width(Length::FillPortion(3)),
                text(format_currency(inc.monthly_amount_cents, self.currency))
                    .size(14)
                    .width(Length::FillPortion(2)),
                text(format!("{:.1}%", inc.annual_growth))
                    .size(14)
                    .width(Length::FillPortion(1)),
                text(period).size(14).width(Length::FillPortion(2)),
                row![
                    button(text("Edit").size(12)).padding(4).on_press(IncomeMessage::Edit(idx)),
                    button(text("Del").size(12)).padding(4).on_press(IncomeMessage::Delete(idx)),
                ]
                .spacing(4)
                .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .padding([6u16, 0u16]);

            list = list.push(item);
        }

        let content = column![title, form, header, scrollable(list)]
            .spacing(16)
            .max_width(900);

        container(content).padding(24).width(Length::Fill).into()
    }
}
