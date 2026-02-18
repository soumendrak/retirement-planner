use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Length};

use retirement_planner_core::currency::Currency;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyOption(pub Currency);

impl std::fmt::Display for CurrencyOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Currency::USD => write!(f, "USD ($)"),
            Currency::INR => write!(f, "INR (₹)"),
        }
    }
}

const CURRENCY_OPTIONS: &[CurrencyOption] = &[
    CurrencyOption(Currency::USD),
    CurrencyOption(Currency::INR),
];

#[derive(Debug, Clone)]
pub enum ProfileMessage {
    NameChanged(String),
    CurrentAgeChanged(String),
    RetirementAgeChanged(String),
    LifeExpectancyChanged(String),
    CurrencySelected(CurrencyOption),
    ExchangeRateChanged(String),
    TaxRateChanged(String),
    Save,
}

#[derive(Debug, Clone)]
pub struct ProfileForm {
    pub name: String,
    pub current_age: String,
    pub retirement_age: String,
    pub life_expectancy: String,
    pub base_currency: Currency,
    pub exchange_rate: String,
    pub effective_tax_rate: String,
    pub saved: bool,
}

impl Default for ProfileForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            current_age: "30".to_string(),
            retirement_age: "60".to_string(),
            life_expectancy: "85".to_string(),
            base_currency: Currency::USD,
            exchange_rate: "83.0".to_string(),
            effective_tax_rate: "25.0".to_string(),
            saved: false,
        }
    }
}

impl ProfileForm {
    pub fn update(&mut self, message: ProfileMessage) {
        self.saved = false;
        match message {
            ProfileMessage::NameChanged(name) => self.name = name,
            ProfileMessage::CurrentAgeChanged(age) => self.current_age = age,
            ProfileMessage::RetirementAgeChanged(age) => self.retirement_age = age,
            ProfileMessage::LifeExpectancyChanged(age) => self.life_expectancy = age,
            ProfileMessage::CurrencySelected(opt) => self.base_currency = opt.0,
            ProfileMessage::ExchangeRateChanged(rate) => self.exchange_rate = rate,
            ProfileMessage::TaxRateChanged(rate) => self.effective_tax_rate = rate,
            ProfileMessage::Save => self.saved = true,
        }
    }

    pub fn view(&self) -> Element<'_, ProfileMessage> {
        let title = text("Profile Setup").size(32);

        let name_field = labeled_input("Name", &self.name, "Your name", ProfileMessage::NameChanged);
        let age_field = labeled_input(
            "Current Age",
            &self.current_age,
            "30",
            ProfileMessage::CurrentAgeChanged,
        );
        let retire_field = labeled_input(
            "Retirement Age",
            &self.retirement_age,
            "60",
            ProfileMessage::RetirementAgeChanged,
        );
        let life_field = labeled_input(
            "Life Expectancy",
            &self.life_expectancy,
            "85",
            ProfileMessage::LifeExpectancyChanged,
        );

        let currency_label = text("Base Currency").size(14);
        let currency_picker = pick_list(
            CURRENCY_OPTIONS,
            Some(CurrencyOption(self.base_currency)),
            ProfileMessage::CurrencySelected,
        )
        .width(Length::Fill);

        let exchange_field = labeled_input(
            "Exchange Rate (1 USD = ? INR)",
            &self.exchange_rate,
            "83.0",
            ProfileMessage::ExchangeRateChanged,
        );

        let tax_field = labeled_input(
            "Effective Tax Rate (%)",
            &self.effective_tax_rate,
            "25.0",
            ProfileMessage::TaxRateChanged,
        );

        let save_btn = button(text("Save Profile").size(16))
            .padding(10)
            .on_press(ProfileMessage::Save);

        let status = if self.saved {
            text("Profile saved!").size(14).color(iced::Color::from_rgb(0.2, 0.7, 0.4))
        } else {
            text("").size(14)
        };

        let form = column![
            title,
            name_field,
            row![
                age_field,
                retire_field,
                life_field,
            ]
            .spacing(16),
            column![currency_label, currency_picker].spacing(4),
            exchange_field,
            tax_field,
            row![save_btn, status].spacing(12).align_y(iced::Alignment::Center),
        ]
        .spacing(16)
        .max_width(600);

        container(form)
            .padding(24)
            .width(Length::Fill)
            .into()
    }
}

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    placeholder: &'a str,
    on_change: impl Fn(String) -> ProfileMessage + 'a,
) -> Element<'a, ProfileMessage> {
    column![
        text(label).size(14),
        text_input(placeholder, value).on_input(on_change).size(16),
    ]
    .spacing(4)
    .into()
}
