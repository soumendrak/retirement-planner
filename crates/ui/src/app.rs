use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use retirement_planner_core::currency::Currency;
use retirement_planner_core::engine::{projection, projection_with_band, summarize_projection, ReturnBand};
use retirement_planner_core::export::{projections_to_csv, PlanExport};
use retirement_planner_core::models::{Profile, Scenario};

use crate::views::dashboard::{DashboardMessage, DashboardView, ScenarioResult};
use crate::views::events::{EventsMessage, EventsView};
use crate::views::expenses::{ExpensesMessage, ExpensesView};
use crate::views::income::{IncomeMessage, IncomeView};
use crate::views::instruments::{InstrumentsMessage, InstrumentsView};
use crate::views::profile::{ProfileForm, ProfileMessage};
use crate::views::scenarios::{ScenariosMessage, ScenariosView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Profile,
    Instruments,
    Expenses,
    Income,
    Events,
    Scenarios,
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Profile => "Profile",
            Page::Instruments => "Instruments",
            Page::Expenses => "Expenses",
            Page::Income => "Income",
            Page::Events => "One-Time Events",
            Page::Scenarios => "Scenarios",
        }
    }

    pub fn all() -> &'static [Page] {
        &[
            Page::Dashboard,
            Page::Profile,
            Page::Instruments,
            Page::Expenses,
            Page::Income,
            Page::Events,
            Page::Scenarios,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    Profile(ProfileMessage),
    Instruments(InstrumentsMessage),
    Expenses(ExpensesMessage),
    Income(IncomeMessage),
    Events(EventsMessage),
    Scenarios(ScenariosMessage),
    Dashboard(DashboardMessage),
    RunProjections,
    ExportJson,
    ImportJson,
    ExportCsv,
}

pub struct App {
    pub current_page: Page,
    pub profile: ProfileForm,
    pub instruments: InstrumentsView,
    pub expenses: ExpensesView,
    pub income: IncomeView,
    pub events: EventsView,
    pub scenarios: ScenariosView,
    pub dashboard: DashboardView,
    pub status_message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_page: Page::Profile,
            profile: ProfileForm::default(),
            instruments: InstrumentsView::default(),
            expenses: ExpensesView::default(),
            income: IncomeView::default(),
            events: EventsView::default(),
            scenarios: ScenariosView::default(),
            dashboard: DashboardView::default(),
            status_message: String::new(),
        }
    }
}

impl App {
    fn current_currency(&self) -> Currency {
        self.profile.base_currency
    }

    fn build_profile(&self) -> Profile {
        Profile {
            id: uuid::Uuid::nil(),
            name: self.profile.name.clone(),
            current_age: self.profile.current_age.parse().unwrap_or(30),
            retirement_age: self.profile.retirement_age.parse().unwrap_or(60),
            life_expectancy: self.profile.life_expectancy.parse().unwrap_or(85),
            base_currency: self.profile.base_currency,
            exchange_rate: self.profile.exchange_rate.parse().unwrap_or(83.0),
            effective_tax_rate: self.profile.effective_tax_rate.parse().unwrap_or(25.0),
        }
    }

    fn build_scenario(&self, name: &str, notes: &str) -> Scenario {
        Scenario {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            profile_id: uuid::Uuid::nil(),
            instruments: self.instruments.instruments.clone(),
            expenses: self.expenses.expenses.clone(),
            incomes: self.income.incomes.clone(),
            inflation_config: self.expenses.inflation_config.clone(),
            notes: notes.to_string(),
            one_time_events: self.events.events.clone(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
            }
            Message::Profile(msg) => {
                self.profile.update(msg);
                // Sync currency to sub-views
                let currency = self.current_currency();
                self.instruments.currency = currency;
                self.expenses.currency = currency;
                self.income.currency = currency;
                self.dashboard.currency = currency;
            }
            Message::Instruments(msg) => self.instruments.update(msg),
            Message::Expenses(msg) => self.expenses.update(msg),
            Message::Income(msg) => self.income.update(msg),
            Message::Events(msg) => {
                self.events.update(msg);
                self.events.currency = self.current_currency();
            }
            Message::Scenarios(msg) => {
                self.scenarios.update(msg);
                // Update scenario entry counts
                for entry in &mut self.scenarios.scenarios {
                    entry.instrument_count = self.instruments.instruments.len();
                    entry.expense_count = self.expenses.expenses.len();
                    entry.income_count = self.income.incomes.len();
                }
            }
            Message::Dashboard(_) => {}
            Message::ExportCsv => {
                if let Some(first) = self.dashboard.results.first() {
                    let symbol = self.current_currency().symbol();
                    let csv = projections_to_csv(&first.projections, symbol);
                    let path = dirs::document_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("retirement_projection.csv");
                    match std::fs::write(&path, &csv) {
                        Ok(_) => self.status_message = format!("CSV exported to {}", path.display()),
                        Err(e) => self.status_message = format!("CSV export failed: {}", e),
                    }
                } else {
                    self.status_message = "Run projections first before exporting CSV".to_string();
                }
            }
            Message::ExportJson => {
                let plan = PlanExport {
                    version: PlanExport::CURRENT_VERSION,
                    profile: self.build_profile(),
                    instruments: self.instruments.instruments.clone(),
                    expenses: self.expenses.expenses.clone(),
                    incomes: self.income.incomes.clone(),
                    one_time_events: self.events.events.clone(),
                    inflation_config: self.expenses.inflation_config.clone(),
                    scenario_names: self.scenarios.scenarios.iter().map(|s| s.name.clone()).collect(),
                };
                match plan.to_json() {
                    Ok(json) => {
                        let path = dirs::document_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("retirement_plan.json");
                        match std::fs::write(&path, &json) {
                            Ok(_) => {
                                self.status_message = format!("Exported to {}", path.display());
                            }
                            Err(e) => {
                                self.status_message = format!("Export failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Serialization failed: {}", e);
                    }
                }
            }
            Message::ImportJson => {
                let path = dirs::document_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("retirement_plan.json");
                match std::fs::read_to_string(&path) {
                    Ok(json) => match PlanExport::from_json(&json) {
                        Ok(plan) => {
                            let errors = plan.validate();
                            if !errors.is_empty() {
                                self.status_message = format!("Validation warnings: {}", errors.join("; "));
                            } else {
                                self.status_message = format!("Imported from {}", path.display());
                            }
                            // Apply imported data
                            self.profile.name = plan.profile.name;
                            self.profile.current_age = plan.profile.current_age.to_string();
                            self.profile.retirement_age = plan.profile.retirement_age.to_string();
                            self.profile.life_expectancy = plan.profile.life_expectancy.to_string();
                            self.profile.base_currency = plan.profile.base_currency;
                            self.profile.exchange_rate = format!("{:.1}", plan.profile.exchange_rate);
                            self.profile.effective_tax_rate = format!("{:.1}", plan.profile.effective_tax_rate);
                            self.instruments.instruments = plan.instruments;
                            self.expenses.expenses = plan.expenses;
                            self.expenses.inflation_config = plan.inflation_config;
                            self.income.incomes = plan.incomes;
                            self.events.events = plan.one_time_events;
                            self.scenarios.scenarios = plan.scenario_names.iter().map(|n| {
                                crate::views::scenarios::ScenarioEntry {
                                    name: n.clone(),
                                    notes: String::new(),
                                    instrument_count: self.instruments.instruments.len(),
                                    expense_count: self.expenses.expenses.len(),
                                    income_count: self.income.incomes.len(),
                                }
                            }).collect();
                            // Sync currency
                            let currency = self.current_currency();
                            self.instruments.currency = currency;
                            self.expenses.currency = currency;
                            self.income.currency = currency;
                            self.dashboard.currency = currency;
                            self.current_page = Page::Profile;
                        }
                        Err(e) => {
                            self.status_message = format!("Import parse error: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Could not read {}: {}", path.display(), e);
                    }
                }
            }
            Message::RunProjections => {
                let profile = self.build_profile();
                let mut results = Vec::new();

                if self.scenarios.scenarios.is_empty() {
                    // Run with default scenario using current data
                    let scenario = self.build_scenario("Current Plan", "");
                    let projections = projection(&profile, &scenario);
                    let summary = summarize_projection(&profile, &projections);
                    results.push(ScenarioResult {
                        name: "Current Plan".to_string(),
                        summary,
                        projections,
                        pessimistic: projection_with_band(&profile, &scenario, ReturnBand::Pessimistic),
                        optimistic: projection_with_band(&profile, &scenario, ReturnBand::Optimistic),
                        instruments: self.instruments.instruments.clone(),
                    });
                } else {
                    for entry in &self.scenarios.scenarios {
                        let scenario = self.build_scenario(&entry.name, &entry.notes);
                        let projections = projection(&profile, &scenario);
                        let summary = summarize_projection(&profile, &projections);
                        results.push(ScenarioResult {
                            name: entry.name.clone(),
                            summary,
                            pessimistic: projection_with_band(&profile, &scenario, ReturnBand::Pessimistic),
                            optimistic: projection_with_band(&profile, &scenario, ReturnBand::Optimistic),
                            projections,
                            instruments: self.instruments.instruments.clone(),
                        });
                    }
                }

                self.dashboard.results = results;
                self.dashboard.currency = self.current_currency();
                self.dashboard.retirement_age = profile.retirement_age;
                self.dashboard.prepare_charts();
                self.current_page = Page::Dashboard;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.sidebar();
        let content = self.content();

        row![sidebar, content].into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let title = text("Retirement\nPlanner")
            .size(20)
            .color(iced::Color::WHITE);

        let mut nav = column![
            container(title).padding([20u16, 16u16]),
        ]
        .spacing(2);

        for page in Page::all() {
            let is_active = *page == self.current_page;

            let label = text(page.label()).size(15).color(if is_active {
                iced::Color::WHITE
            } else {
                iced::Color::from_rgb(0.7, 0.7, 0.75)
            });

            let btn = button(label)
                .padding(10)
                .width(Length::Fill)
                .on_press(Message::NavigateTo(*page))
                .style(move |_theme: &iced::Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(if is_active {
                            iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15)
                        } else {
                            iced::Color::TRANSPARENT
                        })),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

            nav = nav.push(btn);
        }

        // Run projections button at bottom
        let run_btn = button(
            text("Run Projections").size(14).color(iced::Color::WHITE),
        )
        .padding(10)
        .width(Length::Fill)
        .on_press(Message::RunProjections)
        .style(|_theme: &iced::Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.72, 0.4,
            ))),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Export/Import buttons
        let export_btn = button(
            text("Export JSON").size(12).color(iced::Color::from_rgb(0.85, 0.85, 0.9)),
        )
        .padding(8)
        .width(Length::Fill)
        .on_press(Message::ExportJson)
        .style(|_theme: &iced::Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
            text_color: iced::Color::from_rgb(0.85, 0.85, 0.9),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let import_btn = button(
            text("Import JSON").size(12).color(iced::Color::from_rgb(0.85, 0.85, 0.9)),
        )
        .padding(8)
        .width(Length::Fill)
        .on_press(Message::ImportJson)
        .style(|_theme: &iced::Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
            text_color: iced::Color::from_rgb(0.85, 0.85, 0.9),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let csv_btn = button(
            text("Export CSV").size(12).color(iced::Color::from_rgb(0.85, 0.85, 0.9)),
        )
        .padding(8)
        .width(Length::Fill)
        .on_press(Message::ExportCsv)
        .style(|_theme: &iced::Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
            text_color: iced::Color::from_rgb(0.85, 0.85, 0.9),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let io_row = row![export_btn, import_btn].spacing(4);

        let status = if self.status_message.is_empty() {
            text("").size(10)
        } else {
            text(&self.status_message).size(10).color(iced::Color::from_rgb(0.7, 0.85, 0.7))
        };

        let sidebar_content = column![
            nav,
            column![].height(Length::Fill),
            container(
                column![
                    run_btn,
                    io_row,
                    csv_btn,
                    status,
                ]
                .spacing(6)
            )
            .padding(iced::Padding { top: 0.0, right: 8.0, bottom: 12.0, left: 8.0 }),
        ]
        .height(Length::Fill);

        container(sidebar_content)
            .width(220)
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.15, 0.15, 0.20,
                ))),
                ..Default::default()
            })
            .into()
    }

    fn content(&self) -> Element<'_, Message> {
        let page_content: Element<'_, Message> = match self.current_page {
            Page::Dashboard => self.dashboard.view().map(Message::Dashboard),
            Page::Profile => self.profile.view().map(Message::Profile),
            Page::Instruments => self.instruments.view().map(Message::Instruments),
            Page::Expenses => self.expenses.view().map(Message::Expenses),
            Page::Income => self.income.view().map(Message::Income),
            Page::Events => self.events.view().map(Message::Events),
            Page::Scenarios => self.scenarios.view().map(Message::Scenarios),
        };

        container(page_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.97, 0.97, 0.98,
                ))),
                ..Default::default()
            })
            .into()
    }
}
