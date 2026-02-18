use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use retirement_planner_core::currency::{format_currency, Currency};
use retirement_planner_core::models::OneTimeEvent;

#[derive(Debug, Clone)]
pub enum EventsMessage {
    NameChanged(String),
    AgeChanged(String),
    AmountChanged(String),
    NotesChanged(String),
    Add,
    Edit(usize),
    Delete(usize),
    CancelEdit,
    SaveEdit,
}

#[derive(Debug, Clone)]
pub struct EventsView {
    pub events: Vec<OneTimeEvent>,
    pub name: String,
    pub age: String,
    pub amount: String,
    pub notes: String,
    pub editing_index: Option<usize>,
    pub currency: Currency,
}

impl Default for EventsView {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            name: String::new(),
            age: String::new(),
            amount: String::new(),
            notes: String::new(),
            editing_index: None,
            currency: Currency::USD,
        }
    }
}

impl EventsView {
    fn clear_form(&mut self) {
        self.name.clear();
        self.age.clear();
        self.amount.clear();
        self.notes.clear();
        self.editing_index = None;
    }

    pub fn update(&mut self, message: EventsMessage) {
        match message {
            EventsMessage::NameChanged(v) => self.name = v,
            EventsMessage::AgeChanged(v) => self.age = v,
            EventsMessage::AmountChanged(v) => self.amount = v,
            EventsMessage::NotesChanged(v) => self.notes = v,
            EventsMessage::Add => {
                if !self.name.is_empty() && !self.age.is_empty() && !self.amount.is_empty() {
                    let age: u8 = self.age.parse().unwrap_or(0);
                    let amount_cents =
                        (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                    let event = OneTimeEvent {
                        id: uuid::Uuid::new_v4(),
                        name: self.name.clone(),
                        age,
                        amount_cents,
                        notes: self.notes.clone(),
                    };
                    self.events.push(event);
                    self.clear_form();
                }
            }
            EventsMessage::Edit(idx) => {
                if let Some(ev) = self.events.get(idx) {
                    self.name = ev.name.clone();
                    self.age = ev.age.to_string();
                    self.amount = format!("{:.0}", ev.amount_cents as f64 / 100.0);
                    self.notes = ev.notes.clone();
                    self.editing_index = Some(idx);
                }
            }
            EventsMessage::Delete(idx) => {
                if idx < self.events.len() {
                    self.events.remove(idx);
                }
            }
            EventsMessage::CancelEdit => self.clear_form(),
            EventsMessage::SaveEdit => {
                if let Some(idx) = self.editing_index {
                    let name = self.name.clone();
                    let age: u8 = self.age.parse().unwrap_or(0);
                    let amount_cents = (self.amount.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                    let notes = self.notes.clone();
                    if let Some(ev) = self.events.get_mut(idx) {
                        ev.name = name;
                        ev.age = age;
                        ev.amount_cents = amount_cents;
                        ev.notes = notes;
                    }
                }
                self.clear_form();
            }
        }
    }

    pub fn view(&self) -> Element<'_, EventsMessage> {
        let title = text("One-Time Events").size(32);
        let subtitle = text(
            "Model lump-sum inflows (inheritance, bonus) or outflows (home purchase, medical). Positive = inflow, Negative = outflow.",
        )
        .size(13);

        let name_field = column![
            text("Event Name").size(14),
            text_input("e.g. Inheritance, Home Purchase", &self.name)
                .on_input(EventsMessage::NameChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(3));

        let age_field = column![
            text("At Age").size(14),
            text_input("55", &self.age)
                .on_input(EventsMessage::AgeChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(1));

        let amount_field = column![
            text("Amount (+ inflow / - outflow)").size(14),
            text_input("50000 or -200000", &self.amount)
                .on_input(EventsMessage::AmountChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let notes_field = column![
            text("Notes").size(14),
            text_input("Optional", &self.notes)
                .on_input(EventsMessage::NotesChanged)
                .size(16),
        ]
        .spacing(4)
        .width(Length::FillPortion(2));

        let form_buttons = if self.editing_index.is_some() {
            row![
                button(text("Save").size(14)).padding(8).on_press(EventsMessage::SaveEdit),
                button(text("Cancel").size(14)).padding(8).on_press(EventsMessage::CancelEdit),
            ]
            .spacing(8)
        } else {
            row![
                button(text("Add Event").size(14)).padding(8).on_press(EventsMessage::Add),
            ]
        };

        let form = column![
            row![name_field, age_field, amount_field, notes_field].spacing(12),
            form_buttons,
        ]
        .spacing(12);

        let header = row![
            text("Name").size(14).width(Length::FillPortion(3)),
            text("Age").size(14).width(Length::FillPortion(1)),
            text("Amount").size(14).width(Length::FillPortion(2)),
            text("Notes").size(14).width(Length::FillPortion(2)),
            text("Actions").size(14).width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([8u16, 0u16]);

        let mut list = column![].spacing(4);
        for (idx, ev) in self.events.iter().enumerate() {
            let amount_display = format_currency(ev.amount_cents.abs(), self.currency);
            let amount_text = if ev.amount_cents >= 0 {
                format!("+{}", amount_display)
            } else {
                format!("-{}", amount_display)
            };
            let amount_color = if ev.amount_cents >= 0 {
                iced::Color::from_rgb(0.2, 0.72, 0.4)
            } else {
                iced::Color::from_rgb(0.9, 0.25, 0.25)
            };

            let item = row![
                text(&ev.name).size(13).width(Length::FillPortion(3)),
                text(format!("{}", ev.age)).size(13).width(Length::FillPortion(1)),
                text(amount_text).size(13).width(Length::FillPortion(2)).color(amount_color),
                text(if ev.notes.is_empty() { "—" } else { &ev.notes })
                    .size(13)
                    .width(Length::FillPortion(2)),
                row![
                    button(text("Edit").size(12)).padding(4).on_press(EventsMessage::Edit(idx)),
                    button(text("Del").size(12)).padding(4).on_press(EventsMessage::Delete(idx)),
                ]
                .spacing(4)
                .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .padding([3u16, 0u16]);

            list = list.push(item);
        }

        let content = column![title, subtitle, form, header, scrollable(list)]
            .spacing(16)
            .max_width(900);

        container(content).padding(24).width(Length::Fill).into()
    }
}
