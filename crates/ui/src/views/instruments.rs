use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Element, Length};

use retirement_planner_core::models::{AssetClass, Instrument};
use retirement_planner_core::currency::{format_currency, Currency};

#[derive(Debug, Clone)]
pub enum InstrumentsMessage {
    NameChanged(String),
    AssetClassSelected(AssetClassOption),
    ReturnChanged(String),
    CurrentValueChanged(String),
    MonthlyContributionChanged(String),
    Add,
    Edit(usize),
    Delete(usize),
    CancelEdit,
    SaveEdit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetClassOption(pub AssetClass);

impl std::fmt::Display for AssetClassOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

const ASSET_CLASS_OPTIONS: &[AssetClassOption] = &[
    AssetClassOption(AssetClass::Equity),
    AssetClassOption(AssetClass::Debt),
    AssetClassOption(AssetClass::RealEstate),
    AssetClassOption(AssetClass::Cash),
    AssetClassOption(AssetClass::Other),
];

#[derive(Debug, Clone)]
pub struct InstrumentsView {
    pub instruments: Vec<Instrument>,
    pub name: String,
    pub asset_class: AssetClassOption,
    pub expected_return: String,
    pub current_value: String,
    pub monthly_contribution: String,
    pub editing_index: Option<usize>,
    pub currency: Currency,
}

impl Default for InstrumentsView {
    fn default() -> Self {
        Self {
            instruments: Vec::new(),
            name: String::new(),
            asset_class: AssetClassOption(AssetClass::Equity),
            expected_return: "7.0".to_string(),
            current_value: String::new(),
            monthly_contribution: String::new(),
            editing_index: None,
            currency: Currency::USD,
        }
    }
}

impl InstrumentsView {
    fn clear_form(&mut self) {
        self.name.clear();
        self.asset_class = AssetClassOption(AssetClass::Equity);
        self.expected_return = "7.0".to_string();
        self.current_value.clear();
        self.monthly_contribution.clear();
        self.editing_index = None;
    }

    fn parse_cents(&self, s: &str) -> i64 {
        s.parse::<f64>().unwrap_or(0.0).max(0.0) as i64 * 100
    }

    pub fn update(&mut self, message: InstrumentsMessage) {
        match message {
            InstrumentsMessage::NameChanged(n) => self.name = n,
            InstrumentsMessage::AssetClassSelected(ac) => self.asset_class = ac,
            InstrumentsMessage::ReturnChanged(r) => self.expected_return = r,
            InstrumentsMessage::CurrentValueChanged(v) => self.current_value = v,
            InstrumentsMessage::MonthlyContributionChanged(c) => self.monthly_contribution = c,
            InstrumentsMessage::Add => {
                if !self.name.is_empty() {
                    let inst = Instrument {
                        id: uuid::Uuid::new_v4(),
                        name: self.name.clone(),
                        asset_class: self.asset_class.0,
                        expected_annual_return: self.expected_return.parse().unwrap_or(7.0),
                        current_value_cents: self.parse_cents(&self.current_value),
                        monthly_contribution_cents: self.parse_cents(&self.monthly_contribution),
                    };
                    self.instruments.push(inst);
                    self.clear_form();
                }
            }
            InstrumentsMessage::Edit(idx) => {
                if let Some(inst) = self.instruments.get(idx) {
                    self.name = inst.name.clone();
                    self.asset_class = AssetClassOption(inst.asset_class);
                    self.expected_return = format!("{:.1}", inst.expected_annual_return);
                    self.current_value = format!("{:.0}", inst.current_value_cents as f64 / 100.0);
                    self.monthly_contribution =
                        format!("{:.0}", inst.monthly_contribution_cents as f64 / 100.0);
                    self.editing_index = Some(idx);
                }
            }
            InstrumentsMessage::Delete(idx) => {
                if idx < self.instruments.len() {
                    self.instruments.remove(idx);
                }
            }
            InstrumentsMessage::CancelEdit => {
                self.clear_form();
            }
            InstrumentsMessage::SaveEdit => {
                if let Some(idx) = self.editing_index {
                    let name = self.name.clone();
                    let asset_class = self.asset_class.0;
                    let ret = self.expected_return.parse().unwrap_or(7.0);
                    let value = self.parse_cents(&self.current_value);
                    let contrib = self.parse_cents(&self.monthly_contribution);
                    if let Some(inst) = self.instruments.get_mut(idx) {
                        inst.name = name;
                        inst.asset_class = asset_class;
                        inst.expected_annual_return = ret;
                        inst.current_value_cents = value;
                        inst.monthly_contribution_cents = contrib;
                    }
                }
                self.clear_form();
            }
        }
    }

    pub fn view(&self) -> Element<'_, InstrumentsMessage> {
        let title = text("Investment Instruments").size(32);

        // Form
        let name_input = column![
            text("Name").size(14),
            text_input("e.g. US Large Cap Index", &self.name)
                .on_input(InstrumentsMessage::NameChanged)
                .size(16),
        ]
        .spacing(4);

        let class_picker = column![
            text("Asset Class").size(14),
            pick_list(
                ASSET_CLASS_OPTIONS,
                Some(self.asset_class),
                InstrumentsMessage::AssetClassSelected,
            )
            .width(Length::Fill),
        ]
        .spacing(4);

        let return_input = column![
            text("Expected Return (%)").size(14),
            text_input("7.0", &self.expected_return)
                .on_input(InstrumentsMessage::ReturnChanged)
                .size(16),
        ]
        .spacing(4);

        let value_input = column![
            text("Current Value").size(14),
            text_input("10000", &self.current_value)
                .on_input(InstrumentsMessage::CurrentValueChanged)
                .size(16),
        ]
        .spacing(4);

        let contrib_input = column![
            text("Monthly Contribution").size(14),
            text_input("500", &self.monthly_contribution)
                .on_input(InstrumentsMessage::MonthlyContributionChanged)
                .size(16),
        ]
        .spacing(4);

        let form_buttons = if self.editing_index.is_some() {
            row![
                button(text("Save Changes").size(14))
                    .padding(8)
                    .on_press(InstrumentsMessage::SaveEdit),
                button(text("Cancel").size(14))
                    .padding(8)
                    .on_press(InstrumentsMessage::CancelEdit),
            ]
            .spacing(8)
        } else {
            row![
                button(text("Add Instrument").size(14))
                    .padding(8)
                    .on_press(InstrumentsMessage::Add),
            ]
            .spacing(8)
        };

        let form = column![
            row![name_input, class_picker].spacing(16),
            row![return_input, value_input, contrib_input].spacing(16),
            form_buttons,
        ]
        .spacing(12);

        // Instrument list
        let list_header = row![
            text("Name").size(14).width(Length::FillPortion(3)),
            text("Class").size(14).width(Length::FillPortion(2)),
            text("Return").size(14).width(Length::FillPortion(1)),
            text("Value").size(14).width(Length::FillPortion(2)),
            text("Monthly").size(14).width(Length::FillPortion(2)),
            text("Actions").size(14).width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([8u16, 0u16]);

        let mut list_items = column![].spacing(4);
        for (idx, inst) in self.instruments.iter().enumerate() {
            let item = row![
                text(&inst.name).size(14).width(Length::FillPortion(3)),
                text(inst.asset_class.as_str()).size(14).width(Length::FillPortion(2)),
                text(format!("{:.1}%", inst.expected_annual_return))
                    .size(14)
                    .width(Length::FillPortion(1)),
                text(format_currency(inst.current_value_cents, self.currency))
                    .size(14)
                    .width(Length::FillPortion(2)),
                text(format_currency(inst.monthly_contribution_cents, self.currency))
                    .size(14)
                    .width(Length::FillPortion(2)),
                row![
                    button(text("Edit").size(12))
                        .padding(4)
                        .on_press(InstrumentsMessage::Edit(idx)),
                    button(text("Del").size(12))
                        .padding(4)
                        .on_press(InstrumentsMessage::Delete(idx)),
                ]
                .spacing(4)
                .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .padding([6u16, 0u16]);

            list_items = list_items.push(item);
        }

        let content = column![title, form, list_header, scrollable(list_items)]
            .spacing(16)
            .max_width(900);

        container(content).padding(24).width(Length::Fill).into()
    }
}
