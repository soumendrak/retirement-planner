use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum ScenariosMessage {
    NameChanged(String),
    NotesChanged(String),
    Add,
    Select(usize),
    Delete(usize),
    Clone(usize),
}

#[derive(Debug, Clone, Default)]
pub struct ScenarioEntry {
    pub name: String,
    pub notes: String,
    pub instrument_count: usize,
    pub expense_count: usize,
    pub income_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ScenariosView {
    pub scenarios: Vec<ScenarioEntry>,
    pub selected: Option<usize>,
    pub new_name: String,
    pub new_notes: String,
}

impl ScenariosView {
    pub fn update(&mut self, message: ScenariosMessage) {
        match message {
            ScenariosMessage::NameChanged(n) => self.new_name = n,
            ScenariosMessage::NotesChanged(n) => self.new_notes = n,
            ScenariosMessage::Add => {
                if !self.new_name.is_empty() {
                    self.scenarios.push(ScenarioEntry {
                        name: self.new_name.clone(),
                        notes: self.new_notes.clone(),
                        instrument_count: 0,
                        expense_count: 0,
                        income_count: 0,
                    });
                    self.new_name.clear();
                    self.new_notes.clear();
                }
            }
            ScenariosMessage::Select(idx) => {
                self.selected = Some(idx);
            }
            ScenariosMessage::Delete(idx) => {
                if idx < self.scenarios.len() {
                    self.scenarios.remove(idx);
                    if self.selected == Some(idx) {
                        self.selected = None;
                    } else if let Some(sel) = self.selected {
                        if sel > idx {
                            self.selected = Some(sel - 1);
                        }
                    }
                }
            }
            ScenariosMessage::Clone(idx) => {
                if let Some(entry) = self.scenarios.get(idx).cloned() {
                    let mut cloned = entry;
                    cloned.name = format!("{} (copy)", cloned.name);
                    self.scenarios.push(cloned);
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, ScenariosMessage> {
        let title = text("Scenarios").size(32);
        let subtitle = text("Create multiple what-if scenarios to compare side-by-side").size(14);

        // New scenario form
        let form = column![
            text("Create New Scenario").size(18),
            row![
                column![
                    text("Name").size(14),
                    text_input("e.g. Conservative, Aggressive", &self.new_name)
                        .on_input(ScenariosMessage::NameChanged)
                        .size(16),
                ]
                .spacing(4)
                .width(Length::FillPortion(2)),
                column![
                    text("Notes").size(14),
                    text_input("Optional notes", &self.new_notes)
                        .on_input(ScenariosMessage::NotesChanged)
                        .size(16),
                ]
                .spacing(4)
                .width(Length::FillPortion(3)),
            ]
            .spacing(12),
            button(text("Create Scenario").size(14))
                .padding(8)
                .on_press(ScenariosMessage::Add),
        ]
        .spacing(8);

        // Scenario cards
        let mut cards = column![].spacing(12);

        if self.scenarios.is_empty() {
            cards = cards.push(
                container(
                    text("No scenarios yet. Create one to get started!")
                        .size(16),
                )
                .padding(24)
                .width(Length::Fill),
            );
        }

        for (idx, entry) in self.scenarios.iter().enumerate() {
            let is_selected = self.selected == Some(idx);
            let border_color = if is_selected {
                iced::Color::from_rgb(0.25, 0.50, 0.90)
            } else {
                iced::Color::from_rgb(0.85, 0.85, 0.88)
            };

            let card_content = row![
                column![
                    text(&entry.name).size(18),
                    text(if entry.notes.is_empty() {
                        "No notes"
                    } else {
                        &entry.notes
                    })
                    .size(13),
                    text(format!(
                        "{} instruments | {} expenses | {} incomes",
                        entry.instrument_count, entry.expense_count, entry.income_count
                    ))
                    .size(12),
                ]
                .spacing(4)
                .width(Length::Fill),
                column![
                    button(text("Select").size(12))
                        .padding(4)
                        .on_press(ScenariosMessage::Select(idx)),
                    button(text("Clone").size(12))
                        .padding(4)
                        .on_press(ScenariosMessage::Clone(idx)),
                    button(text("Delete").size(12))
                        .padding(4)
                        .on_press(ScenariosMessage::Delete(idx)),
                ]
                .spacing(4),
            ]
            .spacing(16)
            .padding(16);

            let card = container(card_content)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| {
                    container::Style {
                        border: iced::Border {
                            color: border_color,
                            width: if is_selected { 2.0 } else { 1.0 },
                            radius: 8.0.into(),
                        },
                        background: Some(iced::Background::Color(iced::Color::WHITE)),
                        ..Default::default()
                    }
                });

            cards = cards.push(card);
        }

        let content = column![title, subtitle, form, scrollable(cards)]
            .spacing(16)
            .max_width(800);

        container(content).padding(24).width(Length::Fill).into()
    }
}
