use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};

use retirement_planner_core::currency::{format_currency, Currency};
use retirement_planner_core::engine::{ProjectionSummary, YearProjection};
use retirement_planner_core::models::Instrument;

use crate::charts::allocation_pie::AllocationPieChart;
use crate::charts::corpus_line::{CorpusLineChart, ScenarioLine};
use crate::charts::withdrawal_bar::WithdrawalBarChart;

#[derive(Debug, Clone)]
pub enum DashboardMessage {}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub name: String,
    pub summary: ProjectionSummary,
    pub projections: Vec<YearProjection>,
    pub pessimistic: Vec<YearProjection>,
    pub optimistic: Vec<YearProjection>,
    pub instruments: Vec<Instrument>,
}

#[derive(Debug, Clone, Default)]
pub struct DashboardView {
    pub results: Vec<ScenarioResult>,
    pub currency: Currency,
    pub retirement_age: u8,
    pub corpus_chart: CorpusLineChart,
    pub bar_chart: WithdrawalBarChart,
    pub pie_chart: AllocationPieChart,
}

impl DashboardView {
    /// Call after setting results to precompute chart data.
    pub fn prepare_charts(&mut self) {
        let mut scenario_lines: Vec<ScenarioLine> = Vec::new();

        for r in &self.results {
            // Always include the base line
            scenario_lines.push(ScenarioLine {
                name: r.name.clone(),
                projections: r.projections.clone(),
            });
            // Add pessimistic/optimistic bands when multiple bands add value
            if self.results.len() == 1 {
                scenario_lines.push(ScenarioLine {
                    name: format!("{} (Pessimistic)", r.name),
                    projections: r.pessimistic.clone(),
                });
                scenario_lines.push(ScenarioLine {
                    name: format!("{} (Optimistic)", r.name),
                    projections: r.optimistic.clone(),
                });
            }
        }

        self.corpus_chart = CorpusLineChart::new(scenario_lines, self.retirement_age);

        if let Some(first) = self.results.first() {
            self.bar_chart = WithdrawalBarChart::new(
                first.projections.clone(),
                self.retirement_age,
            );
            self.pie_chart = AllocationPieChart::from_instruments(
                &first.instruments,
                format!("Asset Allocation — {}", first.name),
            );
        } else {
            self.bar_chart = WithdrawalBarChart::default();
            self.pie_chart = AllocationPieChart::default();
        }
    }

    pub fn view(&self) -> Element<'_, DashboardMessage> {
        let title = text("Scenario Comparison Dashboard").size(32);

        if self.results.is_empty() {
            let empty = column![
                title,
                container(
                    column![
                        text("No projection results yet").size(20),
                        text("1. Set up your Profile (age, retirement age, currency)").size(14),
                        text("2. Add Investment Instruments").size(14),
                        text("3. Add Monthly Expenses").size(14),
                        text("4. Add Income Sources").size(14),
                        text("5. Create Scenarios (optional)").size(14),
                        text("6. Click \"Run Projections\" in the sidebar").size(14),
                    ]
                    .spacing(8)
                )
                .padding(24)
                .width(Length::Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.85, 0.85, 0.88),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    background: Some(iced::Background::Color(iced::Color::WHITE)),
                    ..Default::default()
                }),
            ]
            .spacing(16)
            .max_width(800);

            return container(empty).padding(24).width(Length::Fill).into();
        }

        // ── Summary cards ──
        let mut cards_row = row![].spacing(12);

        for result in &self.results {
            let card = self.summary_card(&result.name, &result.summary);
            cards_row = cards_row.push(card);
        }

        // ── Corpus line chart (borrows self.corpus_chart) ──
        let corpus_chart_container = self.chart_card(
            self.corpus_chart.view().map(|_: ()| unreachable!()),
            Length::Fill,
        );

        // ── Bottom row: withdrawal bar + allocation pie ──
        let bar_container = self.chart_card(
            self.bar_chart.view().map(|_: ()| unreachable!()),
            Length::FillPortion(3),
        );
        let pie_container = self.chart_card(
            self.pie_chart.view().map(|_: ()| unreachable!()),
            Length::FillPortion(2),
        );
        let bottom_row = row![bar_container, pie_container].spacing(12);

        // ── Year-by-year table ──
        let table = self.projection_table();

        // ── Assemble ──
        let content = column![
            title,
            cards_row,
            horizontal_rule(1),
            corpus_chart_container,
            bottom_row,
            horizontal_rule(1),
            table,
        ]
        .spacing(16)
        .max_width(1000);

        container(scrollable(content))
            .padding(24)
            .width(Length::Fill)
            .into()
    }

    fn chart_card<'a>(
        &self,
        chart_element: Element<'a, DashboardMessage>,
        width: Length,
    ) -> Element<'a, DashboardMessage> {
        container(chart_element)
            .width(width)
            .style(|_theme: &iced::Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.85, 0.85, 0.88),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                ..Default::default()
            })
            .padding(8)
            .into()
    }

    fn summary_card<'a>(
        &'a self,
        name: &'a str,
        s: &ProjectionSummary,
    ) -> Element<'a, DashboardMessage> {
        let depletion_text: Element<'_, DashboardMessage> = if s.corpus_never_depletes {
            row![
                text("Status: ").size(13),
                text("Corpus never depletes")
                    .size(13)
                    .color(iced::Color::from_rgb(0.2, 0.72, 0.4)),
            ]
            .spacing(4)
            .into()
        } else {
            let age = s.depletion_age.unwrap_or(0);
            row![
                text("Status: ").size(13),
                text(format!("Depletes at age {}", age))
                    .size(13)
                    .color(iced::Color::from_rgb(0.9, 0.25, 0.25)),
            ]
            .spacing(4)
            .into()
        };

        let card_content = column![
            text(name).size(16),
            horizontal_rule(1),
            self.metric_row("Corpus at retirement", format_currency(s.corpus_at_retirement_cents, self.currency)),
            self.metric_row("Safe monthly withdrawal", format_currency(s.safe_monthly_withdrawal_cents, self.currency)),
            self.metric_row(
                "Years corpus lasts",
                s.years_corpus_lasts
                    .map(|y| format!("{} years", y))
                    .unwrap_or_else(|| "Forever".to_string()),
            ),
            depletion_text,
        ]
        .spacing(6);

        container(card_content)
            .padding(14)
            .width(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.85, 0.85, 0.88),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                ..Default::default()
            })
            .into()
    }

    fn metric_row<'a>(&self, label: &'a str, value: String) -> Element<'a, DashboardMessage> {
        row![
            text(label).size(13).width(Length::Fill),
            text(value).size(13),
        ]
        .spacing(8)
        .into()
    }

    fn projection_table(&self) -> Element<'_, DashboardMessage> {
        let mut table = column![
            text("Year-by-Year Projection").size(18),
        ]
        .spacing(4);

        if let Some(first) = self.results.first() {
            let header = row![
                text("Age").size(11).width(50),
                text("Corpus").size(11).width(110),
                text("Income").size(11).width(110),
                text("Expenses").size(11).width(110),
                text("Withdrawal").size(11).width(110),
                text("Status").size(11).width(80),
            ]
            .spacing(6)
            .padding([4u16, 0u16]);

            table = table.push(header);
            table = table.push(horizontal_rule(1));

            for proj in first
                .projections
                .iter()
                .filter(|p| p.year % 5 == 0 || p.corpus_cents <= 0)
            {
                let (status, status_color) = if !proj.is_retired {
                    ("Working", iced::Color::from_rgb(0.25, 0.50, 0.90))
                } else if proj.corpus_cents > 0 {
                    ("Retired", iced::Color::from_rgb(0.2, 0.72, 0.4))
                } else {
                    ("Depleted", iced::Color::from_rgb(0.9, 0.25, 0.25))
                };

                let r = row![
                    text(format!("{}", proj.age)).size(11).width(50),
                    text(format_currency(proj.corpus_cents, self.currency))
                        .size(11)
                        .width(110),
                    text(format_currency(proj.total_income_cents, self.currency))
                        .size(11)
                        .width(110),
                    text(format_currency(proj.total_expenses_cents, self.currency))
                        .size(11)
                        .width(110),
                    text(format_currency(proj.net_withdrawal_cents, self.currency))
                        .size(11)
                        .width(110),
                    text(status).size(11).width(80).color(status_color),
                ]
                .spacing(6)
                .padding([2u16, 0u16]);

                table = table.push(r);

                if proj.corpus_cents <= 0 {
                    break;
                }
            }
        }

        container(table)
            .padding(14)
            .width(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.85, 0.85, 0.88),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                ..Default::default()
            })
            .into()
    }
}
