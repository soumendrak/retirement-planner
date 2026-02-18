use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};

use retirement_planner_core::engine::YearProjection;

#[derive(Debug, Clone, Default)]
pub struct WithdrawalBarChart {
    pub projections: Vec<YearProjection>,
    pub retirement_age: u8,
}

impl WithdrawalBarChart {
    pub fn new(projections: Vec<YearProjection>, retirement_age: u8) -> Self {
        Self {
            projections,
            retirement_age,
        }
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(280)
            .into()
    }
}

impl canvas::Program<()> for WithdrawalBarChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        {
            let w = bounds.width;
            let h = bounds.height;
            let ml = 80.0;
            let mr = 20.0;
            let mt = 40.0;
            let mb = 50.0;
            let cw = w - ml - mr;
            let ch = h - mt - mb;

            // Only show post-retirement years
            let retired: Vec<&YearProjection> = self
                .projections
                .iter()
                .filter(|p| p.age >= self.retirement_age && p.corpus_cents >= 0)
                .collect();

            if retired.is_empty() || cw <= 0.0 || ch <= 0.0 {
                return vec![frame.into_geometry()];
            }

            // Show every 2nd year to avoid crowding
            let show: Vec<&&YearProjection> = retired
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, p)| p)
                .collect();

            let max_val = retired
                .iter()
                .map(|p| p.total_expenses_cents.max(p.net_withdrawal_cents))
                .max()
                .unwrap_or(1);

            let bar_count = show.len();
            let bar_total_w = cw / bar_count as f32;
            let bar_w = (bar_total_w * 0.35).min(20.0);
            let gap = bar_w * 0.3;

            // Background
            frame.fill_rectangle(
                Point::new(ml, mt),
                Size::new(cw, ch),
                Color::from_rgb(0.98, 0.98, 0.99),
            );

            // Grid
            for i in 0..=4 {
                let frac = i as f32 / 4.0;
                let y = mt + ch * (1.0 - frac);
                let grid = Path::line(Point::new(ml, y), Point::new(ml + cw, y));
                frame.stroke(
                    &grid,
                    Stroke::default()
                        .with_color(Color::from_rgb(0.88, 0.88, 0.90))
                        .with_width(1.0),
                );

                let val = max_val as f64 * frac as f64 / 100.0;
                let label = if val >= 1_000_000.0 {
                    format!("{:.1}M", val / 1_000_000.0)
                } else if val >= 1_000.0 {
                    format!("{:.0}K", val / 1_000.0)
                } else {
                    format!("{:.0}", val)
                };

                frame.fill_text(Text {
                    content: label,
                    position: Point::new(ml - 8.0, y - 6.0),
                    size: 10.0.into(),
                    color: Color::from_rgb(0.4, 0.4, 0.5),
                    horizontal_alignment: iced::alignment::Horizontal::Right,
                    ..Text::default()
                });
            }

            // Bars
            let expense_color = Color::from_rgb(0.90, 0.40, 0.15);
            let withdrawal_color = Color::from_rgb(0.25, 0.50, 0.90);

            for (i, proj) in show.iter().enumerate() {
                let cx = ml + (i as f32 + 0.5) * bar_total_w;

                // Expense bar
                let exp_h = (proj.total_expenses_cents as f64 / max_val as f64) as f32 * ch;
                frame.fill_rectangle(
                    Point::new(cx - bar_w - gap / 2.0, mt + ch - exp_h),
                    Size::new(bar_w, exp_h),
                    expense_color,
                );

                // Withdrawal bar
                let wd_h = (proj.net_withdrawal_cents as f64 / max_val as f64) as f32 * ch;
                frame.fill_rectangle(
                    Point::new(cx + gap / 2.0, mt + ch - wd_h),
                    Size::new(bar_w, wd_h),
                    withdrawal_color,
                );

                // X label (age)
                frame.fill_text(Text {
                    content: format!("{}", proj.age),
                    position: Point::new(cx, mt + ch + 8.0),
                    size: 10.0.into(),
                    color: Color::from_rgb(0.4, 0.4, 0.5),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    ..Text::default()
                });
            }

            // Legend
            frame.fill_rectangle(
                Point::new(ml + 10.0, mt + 4.0),
                Size::new(12.0, 12.0),
                expense_color,
            );
            frame.fill_text(Text {
                content: "Expenses".to_string(),
                position: Point::new(ml + 26.0, mt + 3.0),
                size: 11.0.into(),
                color: Color::from_rgb(0.2, 0.2, 0.25),
                ..Text::default()
            });

            frame.fill_rectangle(
                Point::new(ml + 100.0, mt + 4.0),
                Size::new(12.0, 12.0),
                withdrawal_color,
            );
            frame.fill_text(Text {
                content: "Withdrawals".to_string(),
                position: Point::new(ml + 116.0, mt + 3.0),
                size: 11.0.into(),
                color: Color::from_rgb(0.2, 0.2, 0.25),
                ..Text::default()
            });

            // Title
            frame.fill_text(Text {
                content: "Expenses vs Withdrawals (Post-Retirement)".to_string(),
                position: Point::new(w / 2.0, 10.0),
                size: 15.0.into(),
                color: Color::from_rgb(0.15, 0.15, 0.2),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                ..Text::default()
            });

            // Axes
            let axes = Path::new(|builder| {
                builder.move_to(Point::new(ml, mt));
                builder.line_to(Point::new(ml, mt + ch));
                builder.line_to(Point::new(ml + cw, mt + ch));
            });
            frame.stroke(
                &axes,
                Stroke::default()
                    .with_color(Color::from_rgb(0.5, 0.5, 0.55))
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }
}
